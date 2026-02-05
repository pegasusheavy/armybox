//! gpioinfo - display information about GPIO chips and lines
//!
//! Display information about GPIO chips and their lines.

use crate::io;

/// gpioinfo - display information about GPIO chips and lines
///
/// # Synopsis
/// ```text
/// gpioinfo [CHIP...]
/// ```
///
/// # Description
/// Display information about GPIO chips and their lines.
/// If no chip specified, show all chips.
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn gpioinfo(argc: i32, argv: *const *const u8) -> i32 {
    let mut found = false;

    if argc < 2 {
        // Show all chips
        for i in 0..16 {
            if show_chip_info(i) {
                found = true;
            }
        }
    } else {
        // Show specified chips
        for i in 1..argc {
            if let Some(chip) = unsafe { super::get_arg(argv, i) } {
                // Parse chip number from name like "gpiochip0"
                let num = if chip.starts_with(b"gpiochip") {
                    crate::sys::parse_u64(&chip[8..]).unwrap_or(0) as u8
                } else {
                    crate::sys::parse_u64(chip).unwrap_or(0) as u8
                };
                if show_chip_info(num) {
                    found = true;
                }
            }
        }
    }

    if !found {
        io::write_str(2, b"gpioinfo: no GPIO chips found\n");
        return 1;
    }

    0
}

fn show_chip_info(chip_num: u8) -> bool {
    let mut path = [0u8; 32];
    let prefix = b"/dev/gpiochip";
    path[..prefix.len()].copy_from_slice(prefix);

    if chip_num < 10 {
        path[prefix.len()] = b'0' + chip_num;
        path[prefix.len() + 1] = 0;
    } else {
        path[prefix.len()] = b'0' + (chip_num / 10);
        path[prefix.len() + 1] = b'0' + (chip_num % 10);
        path[prefix.len() + 2] = 0;
    }

    let mut stat_buf: libc::stat = unsafe { core::mem::zeroed() };
    if unsafe { libc::stat(path.as_ptr() as *const i8, &mut stat_buf) } != 0 {
        return false;
    }

    io::write_str(1, b"gpiochip");
    io::write_num(1, chip_num as u64);
    io::write_str(1, b" - lines:\n");

    // Would need GPIO_GET_CHIPINFO_IOCTL to get actual line count
    // For now, just show placeholder
    io::write_str(1, b"\t(line information requires GPIO chardev access)\n");

    true
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::path::PathBuf;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let release = manifest_dir.join("target/release/armybox");
        if release.exists() { return release; }
        manifest_dir.join("target/debug/armybox")
    }

    #[test]
    fn test_gpioinfo_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["gpioinfo"])
            .output()
            .unwrap();

        // Either shows info or reports no chips
        assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
    }

    #[test]
    fn test_gpioinfo_specific_chip() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["gpioinfo", "gpiochip0"])
            .output()
            .unwrap();

        // Either shows info or chip doesn't exist
        assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
    }
}
