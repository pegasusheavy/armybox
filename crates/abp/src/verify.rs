//! Package Verification
//!
//! Verifies package integrity through checksums and signatures.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use crate::io;
use crate::crypto::{sha256, key_id, PublicKey};
use crate::database::Database;
use crate::format::{AbpPackage, ManifestEntry};

/// Verification result
#[derive(Clone)]
pub struct VerifyResult {
    pub package: String,
    pub signature_valid: bool,
    pub files_ok: usize,
    pub files_missing: Vec<String>,
    pub files_modified: Vec<String>,
}

/// Verify a package file before installation
pub fn verify_package(pkg: &AbpPackage, trusted_keys: &[PublicKey]) -> Result<(), String> {
    // Check signature
    if !verify_package_signature(pkg, trusted_keys)? {
        return Err(String::from("signature verification failed"));
    }

    // Verify internal consistency
    verify_package_integrity(pkg)?;

    Ok(())
}

/// Verify package signature against trusted keys
fn verify_package_signature(pkg: &AbpPackage, trusted_keys: &[PublicKey]) -> Result<bool, String> {
    // Get the signature
    let signature = match &pkg.signature {
        Some(sig) => sig,
        None => return Err(String::from("package has no signature")),
    };

    // Find the signing key
    let sig_key_id = match &pkg.key_id {
        Some(kid) => kid,
        None => return Err(String::from("package has no key ID")),
    };

    for key in trusted_keys {
        let kid = key_id(key.as_bytes());
        if &kid == sig_key_id {
            // Found a matching trusted key. Delegate to the package's own
            // signature check, which now verifies over the exact on-disk bytes
            // captured during parsing (`AbpPackage::signed_region` =
            // header ‖ metadata ‖ manifest ‖ payload, excluding the signature
            // trailer) rather than a re-serialization of the parsed structures.
            //
            // NOTE: the Ed25519 primitive itself (`crypto::verify_signature`)
            // is currently broken — it overflows in field arithmetic (see the
            // ignored RFC 8032 known-answer tests in crypto.rs). Until that is
            // repaired, this returns false for any real signature, so package
            // installation is fail-closed, not actually authenticated.
            let _ = signature;
            return Ok(pkg.verify_signature(key.as_bytes()));
        }
    }

    Err(String::from("signing key not trusted"))
}

/// Verify internal package integrity
///
/// Walks the package's (decompressed) tar payload and, for every regular file
/// member, checks it against the manifest using [`verify_manifest_entry`]. The
/// check fails if the payload contains a file that is not in the manifest
/// (extra), if the manifest lists a file that is not in the payload (missing),
/// or if any file's contents do not match its recorded checksum (mismatch).
fn verify_package_integrity(pkg: &AbpPackage) -> Result<(), String> {
    use crate::install::{parse_tar_name, parse_tar_size};

    let payload = match pkg.decompressed_payload() {
        Some(p) => p,
        None => return Err(String::from("failed to decompress payload")),
    };

    // Track which manifest entries were seen in the payload.
    let mut matched: Vec<bool> = Vec::with_capacity(pkg.manifest.entries.len());
    for _ in 0..pkg.manifest.entries.len() {
        matched.push(false);
    }

    let mut pos = 0;
    while pos + 512 <= payload.len() {
        let header = &payload[pos..pos + 512];

        // End of archive (zero block).
        if header.iter().all(|&b| b == 0) {
            break;
        }

        let name = parse_tar_name(header);
        if name.is_empty() {
            break;
        }

        let size = parse_tar_size(&header[124..136]);
        let typeflag = header[156];

        pos += 512;

        // Only regular files carry checksummed contents.
        let is_regular = typeflag == b'0' || (typeflag == 0 && !name.ends_with(b"/"));

        if is_regular {
            let data_end = pos + size as usize;
            if data_end > payload.len() {
                return Err(String::from("truncated payload"));
            }
            let data = &payload[pos..data_end];

            let norm_name = normalize_member_path(&name);
            let mut found = false;
            for (i, entry) in pkg.manifest.entries.iter().enumerate() {
                if normalize_member_path(entry.path.as_bytes()) == norm_name {
                    if !verify_manifest_entry(entry, data) {
                        return Err(String::from("payload checksum mismatch"));
                    }
                    matched[i] = true;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(String::from("payload contains file not in manifest"));
            }
        }

        // Advance past this member's data (rounded up to a 512-byte boundary).
        pos += ((size + 511) / 512 * 512) as usize;
    }

    // Any manifest entry not encountered in the payload is missing.
    for (i, entry) in pkg.manifest.entries.iter().enumerate() {
        if !matched[i] {
            let mut msg = String::from("manifest entry missing from payload: ");
            msg.push_str(&entry.path);
            return Err(msg);
        }
    }

    Ok(())
}

/// Normalise a member/manifest path for comparison by stripping any leading
/// `./` sequences and leading `/` characters, so equivalent relative and
/// rooted spellings compare equal.
fn normalize_member_path(path: &[u8]) -> &[u8] {
    let mut p = path;
    loop {
        if p.starts_with(b"./") {
            p = &p[2..];
        } else if p.first() == Some(&b'/') {
            p = &p[1..];
        } else {
            break;
        }
    }
    p
}

/// Verify installed packages
pub fn verify_installed(packages: &[&[u8]]) -> i32 {
    let db = match Database::open() {
        Some(db) => db,
        None => {
            io::write_str(2, b"abp: cannot open database\n");
            return 1;
        }
    };

    let to_verify: Vec<String> = if packages.is_empty() {
        // Verify all installed packages
        db.list_packages().iter().map(|p| p.name.clone()).collect()
    } else {
        // Verify specified packages
        packages.iter()
            .filter_map(|p| core::str::from_utf8(p).ok())
            .map(String::from)
            .collect()
    };

    let mut total_ok = 0;
    let mut total_missing = 0;
    let mut total_modified = 0;
    let mut errors = 0;

    for name in &to_verify {
        let pkg = match db.get_package(name) {
            Some(p) => p,
            None => {
                io::write_str(2, b"abp: package '");
                io::write_all(2, name.as_bytes());
                io::write_str(2, b"' not installed\n");
                errors += 1;
                continue;
            }
        };

        io::write_str(1, b"Verifying ");
        io::write_all(1, name.as_bytes());
        io::write_str(1, b"-");
        io::write_all(1, pkg.version.as_bytes());
        io::write_str(1, b"... ");

        // Note: checksums are not stored in PackageRecord - verify file existence only
        let result = verify_package_files(&pkg.name, &pkg.files, &[]);

        if result.files_missing.is_empty() && result.files_modified.is_empty() {
            io::write_str(1, b"OK\n");
            total_ok += result.files_ok;
        } else {
            io::write_str(1, b"FAILED\n");

            for file in &result.files_missing {
                io::write_str(1, b"  Missing: ");
                io::write_all(1, file.as_bytes());
                io::write_str(1, b"\n");
            }

            for file in &result.files_modified {
                io::write_str(1, b"  Modified: ");
                io::write_all(1, file.as_bytes());
                io::write_str(1, b"\n");
            }

            total_missing += result.files_missing.len();
            total_modified += result.files_modified.len();
            errors += 1;
        }
    }

    io::write_str(1, b"\nVerification summary:\n");
    io::write_str(1, b"  Files OK: ");
    io::write_num(1, total_ok as u64);
    io::write_str(1, b"\n");

    if total_missing > 0 {
        io::write_str(1, b"  Files missing: ");
        io::write_num(1, total_missing as u64);
        io::write_str(1, b"\n");
    }

    if total_modified > 0 {
        io::write_str(1, b"  Files modified: ");
        io::write_num(1, total_modified as u64);
        io::write_str(1, b"\n");
    }

    if errors > 0 {
        1
    } else {
        0
    }
}

/// Verify files of an installed package
fn verify_package_files(
    _name: &str,
    files: &[String],
    checksums: &[(String, [u8; 32])],
) -> VerifyResult {
    let mut result = VerifyResult {
        package: String::new(),
        signature_valid: true,
        files_ok: 0,
        files_missing: Vec::new(),
        files_modified: Vec::new(),
    };

    // Build checksum map
    let checksum_map: alloc::collections::BTreeMap<&str, &[u8; 32]> = checksums
        .iter()
        .map(|(path, hash)| (path.as_str(), hash))
        .collect();

    for file in files {
        let path = file.as_bytes();

        // Check if file exists
        if io::access(path, libc::F_OK) != 0 {
            result.files_missing.push(file.clone());
            continue;
        }

        // Check if we have a checksum for this file
        if let Some(&expected) = checksum_map.get(file.as_str()) {
            // Compute actual checksum
            match compute_file_checksum(path) {
                Some(actual) => {
                    if &actual == expected {
                        result.files_ok += 1;
                    } else {
                        result.files_modified.push(file.clone());
                    }
                }
                None => {
                    // Could be a directory or unreadable
                    result.files_ok += 1;
                }
            }
        } else {
            // No checksum recorded, assume OK
            result.files_ok += 1;
        }
    }

    result
}

/// Compute SHA256 checksum of a file
fn compute_file_checksum(path: &[u8]) -> Option<[u8; 32]> {
    let fd = io::open(path, libc::O_RDONLY, 0);
    if fd < 0 {
        return None;
    }

    // Check if it's a regular file
    let mut stat_buf = io::stat_zeroed();
    if io::fstat(fd, &mut stat_buf) != 0 {
        io::close(fd);
        return None;
    }

    if (stat_buf.st_mode & libc::S_IFMT) != libc::S_IFREG {
        io::close(fd);
        return None;
    }

    // Read file and compute hash
    let mut data = Vec::new();
    let mut buf = [0u8; 4096];

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 {
            break;
        }
        data.extend_from_slice(&buf[..n as usize]);
    }

    io::close(fd);

    Some(sha256(&data))
}

/// Load trusted public keys from /etc/abp/keys/
///
/// Enumerates the directory and reads each `*.pub` file. Each file may hold
/// either a raw 32-byte ed25519 public key or a 64-character hex encoding of
/// one (with optional surrounding whitespace). Malformed keys and keys that do
/// not lie on the curve are skipped.
pub fn load_trusted_keys() -> Vec<PublicKey> {
    let mut keys = Vec::new();
    let keys_dir: &[u8] = b"/etc/abp/keys";

    let dir = io::opendir(keys_dir);
    if dir.is_null() {
        return keys;
    }

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() {
            break;
        }

        let (name, _len) = unsafe { io::dirent_name(entry) };

        // Only consider *.pub files (this also skips "." and "..").
        if !name.ends_with(b".pub") {
            continue;
        }

        // Build the full path: /etc/abp/keys/<name>
        let mut path = Vec::with_capacity(keys_dir.len() + 1 + name.len());
        path.extend_from_slice(keys_dir);
        path.push(b'/');
        path.extend_from_slice(name);

        let fd = io::open(&path, libc::O_RDONLY, 0);
        if fd < 0 {
            continue;
        }
        let data = io::read_all(fd);
        io::close(fd);

        if let Some(key) = parse_public_key(&data) {
            keys.push(key);
        }
    }

    io::closedir(dir);
    keys
}

/// Trim leading and trailing ASCII whitespace from a byte slice.
fn trim_ascii_ws(data: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = data.len();
    while start < end && data[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && data[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &data[start..end]
}

/// Parse a public key from the raw bytes of a `.pub` file.
///
/// Accepts an exact 32-byte raw key, a 32-byte raw key surrounded by
/// whitespace, or a 64-character hex encoding. Returns `None` for anything
/// that does not parse into a valid on-curve ed25519 public key.
fn parse_public_key(data: &[u8]) -> Option<PublicKey> {
    // Exact raw 32-byte key (avoid trimming first, so a binary key whose
    // first/last byte happens to look like whitespace is not corrupted).
    if data.len() == 32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(data);
        return PublicKey::from_bytes(&bytes);
    }

    let trimmed = trim_ascii_ws(data);

    // Raw 32-byte key with surrounding whitespace (e.g. a trailing newline).
    if trimmed.len() == 32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(trimmed);
        return PublicKey::from_bytes(&bytes);
    }

    // Hex-encoded 32-byte key (64 hex characters).
    if trimmed.len() == 64 {
        let decoded = crate::crypto::from_hex(trimmed)?;
        if decoded.len() == 32 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&decoded);
            return PublicKey::from_bytes(&bytes);
        }
    }

    None
}

/// Verify a manifest entry
pub fn verify_manifest_entry(entry: &ManifestEntry, data: &[u8]) -> bool {
    let actual = sha256(data);
    actual == entry.checksum
}

/// Generate manifest entries for files
pub fn generate_manifest(files: &[(String, Vec<u8>)]) -> Vec<ManifestEntry> {
    files
        .iter()
        .map(|(path, data)| ManifestEntry {
            path: path.clone(),
            checksum: sha256(data),
            size: data.len() as u64,
            mode: 0o644,
        })
        .collect()
}
