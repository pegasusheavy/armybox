//! # Armybox - A `#[no_std]` BusyBox Clone
//!
//! Armybox provides Unix utilities in a tiny, `#[no_std]` compatible binary.
//! It uses only `libc` for system calls and `alloc` for heap allocation.
//!
//! ## Features
//!
//! - **Truly `#[no_std]`**: No standard library dependency
//! - **Tiny binary**: ~74KB release, ~33KB with UPX compression
//! - **Embedded-ready**: Works on systems without full std support

// Use no_std except during tests (which require std for test harness)
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod io;
pub mod applets;
pub mod sys;

/// Applet function type
pub type AppletFn = fn(i32, *const *const u8) -> i32;

/// Check if an applet exists
pub fn is_applet(name: &[u8]) -> bool {
    applets::find_applet(name).is_some()
}

/// Run an applet by name
pub fn run_applet(name: &[u8], argc: i32, argv: *const *const u8) -> i32 {
    match applets::find_applet(name) {
        Some(f) => f(argc, argv),
        None => {
            io::write_str(2, b"armybox: applet not found: ");
            io::write_all(2, name);
            io::write_str(2, b"\n");
            127
        }
    }
}

/// Get applet count
pub const fn applet_count() -> usize {
    applets::APPLET_COUNT
}

// ============================================================================
// Panic handler for no_std (not used in test builds)
// ============================================================================

#[cfg(all(not(test), not(feature = "std")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    io::write_str(2, b"armybox: panic!\n");
    unsafe { libc::_exit(1); }
}

// ============================================================================
// Global allocator using libc malloc (not used in test builds)
// ============================================================================

#[cfg(all(not(test), not(feature = "std"), feature = "alloc"))]
mod allocator {
    use core::alloc::{GlobalAlloc, Layout};

    pub struct LibcAllocator;

    unsafe impl GlobalAlloc for LibcAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            unsafe { libc::malloc(layout.size()) as *mut u8 }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            unsafe { libc::free(ptr as *mut libc::c_void) }
        }

        unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
            unsafe { libc::realloc(ptr as *mut libc::c_void, new_size) as *mut u8 }
        }
    }

    #[global_allocator]
    static ALLOCATOR: LibcAllocator = LibcAllocator;
}
