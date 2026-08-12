//! Package Installation
//!
//! Implements the `abp add` command for installing packages.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use crate::io;
use crate::format::AbpPackage;
use crate::database::{Database, PackageRecord, current_time};
use crate::resolver::resolve_with_local;

/// Install packages
pub fn install_packages(packages: &[&[u8]], force: bool, no_deps: bool) -> i32 {
    let db = match Database::open() {
        Some(db) => db,
        None => {
            io::write_str(2, b"abp: cannot open database\n");
            return 1;
        }
    };

    let mut to_install: Vec<AbpPackage> = Vec::new();

    // Load all packages
    for &pkg_path in packages {
        // Check if it's a local file or package name
        if pkg_path.contains(&b'/') || pkg_path.ends_with(b".abp") {
            // Local file
            match AbpPackage::from_file(pkg_path) {
                Some(pkg) => {
                    io::write_str(1, b"Loading ");
                    io::write_all(1, pkg_path);
                    io::write_str(1, b"... ");
                    io::write_str(1, b"OK\n");
                    to_install.push(pkg);
                }
                None => {
                    io::write_str(2, b"abp: cannot read package ");
                    io::write_all(2, pkg_path);
                    io::write_str(2, b"\n");
                    return 1;
                }
            }
        } else {
            // Package name - look in available packages
            let name = match core::str::from_utf8(pkg_path) {
                Ok(s) => s,
                Err(_) => {
                    io::write_str(2, b"abp: invalid package name\n");
                    return 1;
                }
            };

            if let Some(avail) = db.get_available(name) {
                io::write_str(1, b"Package '");
                io::write_all(1, pkg_path);
                io::write_str(1, b"' found in repository '");
                io::write_all(1, avail.repo.as_bytes());
                io::write_str(1, b"'\n");

                // TODO: Download package from repository
                io::write_str(2, b"abp: downloading from repositories not yet implemented\n");
                io::write_str(2, b"abp: please specify a local .abp file\n");
                return 1;
            } else {
                io::write_str(2, b"abp: package '");
                io::write_all(2, pkg_path);
                io::write_str(2, b"' not found\n");
                return 1;
            }
        }
    }

    if to_install.is_empty() {
        io::write_str(2, b"abp: no packages to install\n");
        return 1;
    }

    // Resolve dependencies
    let install_order = if no_deps {
        to_install.iter().map(|p| p.metadata.name.clone()).collect()
    } else {
        let pkg_names: Vec<String> = to_install.iter()
            .map(|p| p.metadata.name.clone())
            .collect();

        // Build a map of local packages being installed (for the resolver)
        let mut local_deps: Vec<(String, Vec<String>)> = Vec::new();
        for pkg in &to_install {
            local_deps.push((pkg.metadata.name.clone(), pkg.metadata.depends.clone()));
        }

        match resolve_with_local(&db, &pkg_names, &local_deps) {
            Ok(order) => order,
            Err(e) => {
                io::write_str(2, b"abp: dependency resolution failed: ");
                io::write_all(2, e.as_bytes());
                io::write_str(2, b"\n");
                return 1;
            }
        }
    };

    // Check for conflicts
    if !force {
        for pkg in &to_install {
            // Check if already installed
            if db.is_installed(&pkg.metadata.name) {
                let installed = db.get_package(&pkg.metadata.name).unwrap();
                if installed.version == pkg.metadata.version {
                    io::write_str(1, b"Package '");
                    io::write_all(1, pkg.metadata.name.as_bytes());
                    io::write_str(1, b"' is already installed\n");
                    continue;
                }
                io::write_str(1, b"Upgrading ");
                io::write_all(1, pkg.metadata.name.as_bytes());
                io::write_str(1, b" from ");
                io::write_all(1, installed.version.as_bytes());
                io::write_str(1, b" to ");
                io::write_all(1, pkg.metadata.version.as_bytes());
                io::write_str(1, b"\n");
            }

            // Check file conflicts
            for entry in &pkg.manifest.entries {
                if let Some(owner) = db.get_file_owner(&entry.path) {
                    if owner != pkg.metadata.name {
                        io::write_str(2, b"abp: file conflict: ");
                        io::write_all(2, entry.path.as_bytes());
                        io::write_str(2, b" is owned by ");
                        io::write_all(2, owner.as_bytes());
                        io::write_str(2, b"\n");
                        io::write_str(2, b"abp: use --force to override\n");
                        return 1;
                    }
                }
            }
        }
    }

    // Show transaction summary
    io::write_str(1, b"\nPackages to install:\n");
    for pkg in &to_install {
        io::write_str(1, b"  ");
        io::write_all(1, pkg.metadata.name.as_bytes());
        io::write_str(1, b"-");
        io::write_all(1, pkg.metadata.version.as_bytes());
        if db.is_installed(&pkg.metadata.name) {
            io::write_str(1, b" [upgrade]");
        }
        io::write_str(1, b"\n");
    }
    io::write_str(1, b"\n");

    // Order the loaded packages according to the resolved dependency order
    // (dependencies first) rather than command-line order. Packages named in
    // the resolved order that are not among the locally loaded packages (e.g.
    // dependencies that would be downloaded) are skipped here.
    let mut ordered: Vec<&AbpPackage> = Vec::new();
    for name in &install_order {
        if let Some(pkg) = to_install.iter().find(|p| &p.metadata.name == name) {
            if !ordered.iter().any(|p| p.metadata.name == pkg.metadata.name) {
                ordered.push(pkg);
            }
        }
    }
    // Safety net: append any loaded package not covered by the resolved order.
    for pkg in &to_install {
        if !ordered.iter().any(|p| p.metadata.name == pkg.metadata.name) {
            ordered.push(pkg);
        }
    }

    // Load trusted keys once for signature verification.
    let trusted_keys = crate::verify::load_trusted_keys();

    // Execute installation
    let mut success = true;
    for pkg in ordered {
        // Verify the package (signature + integrity) before extracting.
        // Packages that fail verification are skipped and mark the whole
        // transaction as failed, but do not abort remaining packages.
        if let Err(e) = crate::verify::verify_package(pkg, &trusted_keys) {
            io::write_str(2, b"abp: verification failed for ");
            io::write_all(2, pkg.metadata.name.as_bytes());
            io::write_str(2, b": ");
            io::write_all(2, e.as_bytes());
            io::write_str(2, b"\n");
            io::write_str(2, b"abp: skipping unverified package\n");
            success = false;
            continue;
        }

        if !install_single_package(&db, pkg, force) {
            success = false;
            break;
        }
    }

    if success {
        io::write_str(1, b"\nInstallation complete.\n");
        0
    } else {
        io::write_str(2, b"\nInstallation failed.\n");
        1
    }
}

/// Install a single package
fn install_single_package(db: &Database, pkg: &AbpPackage, _force: bool) -> bool {
    io::write_str(1, b"Installing ");
    io::write_all(1, pkg.metadata.name.as_bytes());
    io::write_str(1, b"-");
    io::write_all(1, pkg.metadata.version.as_bytes());
    io::write_str(1, b"...\n");

    // If upgrading, remove old files first
    if let Some(old_pkg) = db.get_package(&pkg.metadata.name) {
        io::write_str(1, b"  Removing old version...\n");
        for file in &old_pkg.files {
            let path = file.as_bytes();
            io::unlink(path);
        }
        db.unregister_files(&pkg.metadata.name);
    }

    // Extract payload (tar archive, possibly zstd-compressed)
    io::write_str(1, b"  Extracting files...\n");

    let payload_data = match pkg.decompressed_payload() {
        Some(data) => data,
        None => {
            io::write_str(2, b"abp: failed to decompress payload\n");
            return false;
        }
    };

    let files_extracted = extract_payload(&payload_data, b"/", &pkg.manifest);
    if !files_extracted {
        io::write_str(2, b"abp: failed to extract package\n");
        return false;
    }

    // Register files in database. A failed registration leaves the on-disk
    // state and the database out of sync, so treat it as an install failure.
    for entry in &pkg.manifest.entries {
        if !db.register_file(&entry.path, &pkg.metadata.name) {
            io::write_str(2, b"abp: failed to register file: ");
            io::write_all(2, entry.path.as_bytes());
            io::write_str(2, b"\n");
            return false;
        }
    }

    // Create package record
    let record = PackageRecord::from_metadata(&pkg.metadata, &pkg.manifest, current_time());
    if !db.put_package(&record) {
        io::write_str(2, b"abp: failed to register package\n");
        return false;
    }

    io::write_str(1, b"  Done.\n");
    true
}

/// Extract tar payload to root directory
/// The payload should already be decompressed (zstd decompression happens in caller)
fn extract_payload(payload: &[u8], root: &[u8], _manifest: &super::format::Manifest) -> bool {

    let mut pos = 0;
    let mut success = true;

    while pos + 512 <= payload.len() {
        // Read tar header
        let header = &payload[pos..pos + 512];

        // Check for end of archive (two zero blocks)
        if header.iter().all(|&b| b == 0) {
            break;
        }

        // Parse tar header
        let name = parse_tar_name(header);
        if name.is_empty() {
            break;
        }

        let size = parse_tar_size(&header[124..136]);
        let typeflag = header[156];
        // Strip the setuid/setgid/sticky bits (S_ISUID|S_ISGID|S_ISVTX =
        // 0o7000) from the archived mode before it reaches open()/mkdir().
        // Honoring them would let a package drop a root-owned setuid binary on
        // the system. Only the permission bits are trusted.
        let mode = parse_tar_mode(&header[100..108]) & !0o7000;

        // Number of bytes this member's data occupies in the archive, rounded
        // up to a 512-byte block. Non-regular members (dirs, symlinks, unknown
        // types) are normally zero-length, but a hostile archive can set a
        // nonzero size on them to desync the walk unless we always advance.
        let data_blocks = ((size + 511) / 512 * 512) as usize;

        pos += 512; // Move past header

        // Reject path-traversal attempts: absolute member names or names that
        // contain a ".." path component would let a package write outside the
        // installation root. Skip the member, record failure, and advance past
        // any file data.
        if is_unsafe_member_name(&name) {
            io::write_str(2, b"abp: refusing unsafe path in package: ");
            io::write_all(2, &name);
            io::write_str(2, b"\n");
            success = false;
            pos += data_blocks;
            continue;
        }

        // Build full path
        let mut full_path = root.to_vec();
        if !full_path.ends_with(b"/") {
            full_path.push(b'/');
        }
        full_path.extend_from_slice(&name);

        match typeflag {
            // A '5' typeflag is a directory regardless of a trailing slash.
            b'5' => {
                if !mkdir_ok(&full_path, mode) {
                    io::write_str(2, b"abp: failed to create directory: ");
                    io::write_all(2, &full_path);
                    io::write_str(2, b"\n");
                    success = false;
                }
            }
            // A legacy '\0' typeflag with a trailing slash is also a directory.
            0 if name.ends_with(b"/") => {
                if !mkdir_ok(&full_path, mode) {
                    io::write_str(2, b"abp: failed to create directory: ");
                    io::write_all(2, &full_path);
                    io::write_str(2, b"\n");
                    success = false;
                }
            }
            b'0' | 0 => {
                // Regular file
                if !create_parent_dirs(&full_path) {
                    io::write_str(2, b"abp: failed to create parent directories for: ");
                    io::write_all(2, &full_path);
                    io::write_str(2, b"\n");
                    success = false;
                }

                // O_NOFOLLOW refuses to open through an existing symlink at the
                // final path component: if a hostile earlier member planted a
                // symlink here, the open fails (ELOOP) instead of writing
                // through it and escaping the install root.
                let fd = io::open(
                    &full_path,
                    libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC | libc::O_NOFOLLOW,
                    mode,
                );
                if fd >= 0 {
                    let data_end = pos + size as usize;
                    if data_end <= payload.len() {
                        let expected = (data_end - pos) as isize;
                        let written = io::write_all(fd, &payload[pos..data_end]);
                        if written != expected {
                            io::write_str(2, b"abp: failed to write file: ");
                            io::write_all(2, &full_path);
                            io::write_str(2, b"\n");
                            success = false;
                        }
                    } else {
                        // Payload is truncated: the header claims more data
                        // than the archive actually contains.
                        io::write_str(2, b"abp: truncated payload for: ");
                        io::write_all(2, &full_path);
                        io::write_str(2, b"\n");
                        success = false;
                    }
                    io::close(fd);
                } else {
                    // A failure here includes ELOOP from O_NOFOLLOW (an existing
                    // symlink at this path). Treat any open failure as a hard
                    // error rather than silently succeeding.
                    io::write_str(2, b"abp: failed to create file: ");
                    io::write_all(2, &full_path);
                    io::write_str(2, b"\n");
                    success = false;
                }
            }
            b'2' => {
                // Symlink. Validate the TARGET (linkname) with the same rules as
                // member names: an absolute target or one containing a ".."
                // component could point the link outside the install root, so a
                // later write through it (e.g. `x -> /etc`, then `x/cron.d/...`)
                // would escape as root. Reject such symlinks entirely.
                let linkname = parse_tar_name(&header[157..257]);
                if is_unsafe_member_name(&linkname) {
                    io::write_str(2, b"abp: refusing unsafe symlink target in package: ");
                    io::write_all(2, &linkname);
                    io::write_str(2, b"\n");
                    success = false;
                } else if !linkname.is_empty() {
                    if !create_parent_dirs(&full_path) {
                        io::write_str(2, b"abp: failed to create parent directories for: ");
                        io::write_all(2, &full_path);
                        io::write_str(2, b"\n");
                        success = false;
                    }
                    if io::symlink(&linkname, &full_path) != 0 {
                        io::write_str(2, b"abp: failed to create symlink: ");
                        io::write_all(2, &full_path);
                        io::write_str(2, b"\n");
                        success = false;
                    }
                }
            }
            _ => {
                // Unknown/unsupported member type: nothing to create.
            }
        }

        // Advance past this member's data for EVERY member type. Directories,
        // symlinks and unknown types are normally zero-length, but always
        // advancing by the declared (block-rounded) size keeps the walk in sync
        // even if a hostile archive attaches data to a non-regular member.
        pos += data_blocks;
    }

    success
}

/// Return true if a tar member name is unsafe to extract: an absolute path
/// (leading `/`) or one containing a `..` path component. Such names could be
/// used to escape the installation root (path traversal).
fn is_unsafe_member_name(name: &[u8]) -> bool {
    if name.first() == Some(&b'/') {
        return true;
    }
    for component in name.split(|&b| b == b'/') {
        if component == b".." {
            return true;
        }
    }
    false
}

pub(crate) fn parse_tar_name(header: &[u8]) -> Vec<u8> {
    let end = header.iter().position(|&b| b == 0).unwrap_or(header.len());
    header[..end].to_vec()
}

pub(crate) fn parse_tar_size(field: &[u8]) -> u64 {
    let mut result = 0u64;
    for &b in field {
        if b == 0 || b == b' ' {
            break;
        }
        if b >= b'0' && b <= b'7' {
            result = result * 8 + (b - b'0') as u64;
        }
    }
    result
}

fn parse_tar_mode(field: &[u8]) -> u32 {
    let mut result = 0u32;
    for &b in field {
        if b == 0 || b == b' ' {
            break;
        }
        if b >= b'0' && b <= b'7' {
            result = result * 8 + (b - b'0') as u32;
        }
    }
    result
}

/// Current value of the C `errno`.
fn errno() -> i32 {
    unsafe { *libc::__errno_location() }
}

/// Create a directory, treating an already-existing directory as success.
/// Returns `false` only for a genuine failure (a non-EEXIST error), so callers
/// can propagate real problems without tripping over pre-existing directories.
fn mkdir_ok(path: &[u8], mode: u32) -> bool {
    if io::mkdir(path, mode) == 0 {
        return true;
    }
    errno() == libc::EEXIST
}

/// Create every parent directory of `path`. Returns `false` if any component
/// could not be created (ignoring already-existing directories), so the caller
/// can mark the extraction as failed instead of writing into a broken tree.
fn create_parent_dirs(path: &[u8]) -> bool {
    let mut ok = true;
    for i in 0..path.len() {
        if path[i] == b'/' && i > 0 {
            let parent = &path[..i];
            if !mkdir_ok(parent, 0o755) {
                ok = false;
            }
        }
    }
    ok
}

/// Upgrade all installed packages
pub fn upgrade_all(dry_run: bool) -> i32 {
    let db = match Database::open() {
        Some(db) => db,
        None => {
            io::write_str(2, b"abp: cannot open database\n");
            return 1;
        }
    };

    let installed = db.list_packages();
    let mut upgradeable = Vec::new();

    for pkg in &installed {
        if let Some(avail) = db.get_available(&pkg.name) {
            use crate::format::Version;
            let installed_ver = Version::parse(&pkg.version);
            let available_ver = Version::parse(&avail.version);

            if available_ver.compare(&installed_ver) == core::cmp::Ordering::Greater {
                upgradeable.push((pkg.clone(), avail));
            }
        }
    }

    if upgradeable.is_empty() {
        io::write_str(1, b"All packages are up to date.\n");
        return 0;
    }

    io::write_str(1, b"Packages to upgrade:\n");
    for (pkg, avail) in &upgradeable {
        io::write_str(1, b"  ");
        io::write_all(1, pkg.name.as_bytes());
        io::write_str(1, b" ");
        io::write_all(1, pkg.version.as_bytes());
        io::write_str(1, b" -> ");
        io::write_all(1, avail.version.as_bytes());
        io::write_str(1, b"\n");
    }

    if dry_run {
        io::write_str(1, b"\n(dry run - no changes made)\n");
        return 0;
    }

    io::write_str(2, b"\nabp: upgrade from repositories not yet implemented\n");
    io::write_str(2, b"abp: please use 'abp add <package.abp>' to upgrade manually\n");
    1
}
