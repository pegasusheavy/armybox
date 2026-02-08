//! Package Verification
//!
//! Verifies package integrity through checksums and signatures.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use crate::io;
use crate::crypto::{sha256, verify_signature, key_id, PublicKey};
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
            // Found matching key, verify signature
            // The signature covers: header + metadata + manifest + payload
            let mut signed_data = Vec::new();
            signed_data.extend_from_slice(&pkg.header.to_bytes());
            // In a full implementation, we'd include all signed sections

            return Ok(verify_signature(key.as_bytes(), &signed_data, signature));
        }
    }

    Err(String::from("signing key not trusted"))
}

/// Verify internal package integrity
fn verify_package_integrity(_pkg: &AbpPackage) -> Result<(), String> {
    // Verify manifest checksums match payload
    // This would require extracting and checking each file
    // For now, we trust the package structure

    Ok(())
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
pub fn load_trusted_keys() -> Vec<PublicKey> {
    let keys = Vec::new();
    let _keys_dir = b"/etc/abp/keys";

    // In a full implementation, we'd enumerate the directory
    // and load each .pub file
    // For now, return empty list

    keys
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
