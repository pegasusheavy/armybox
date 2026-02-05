//! Shell state management
//!
//! This module contains the `Shell` struct which maintains the runtime state
//! of the shell, including exit status, mode flags, and local variables.

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Shell state
#[cfg(feature = "alloc")]
pub(super) struct Shell {
    /// Last exit status
    pub(super) last_status: i32,
    /// Interactive mode
    pub(super) interactive: bool,
    /// Should exit
    pub(super) should_exit: bool,
    /// Exit code to return
    pub(super) exit_code: i32,
    /// Local variables (not exported)
    pub(super) variables: BTreeMap<Vec<u8>, Vec<u8>>,
}

#[cfg(feature = "alloc")]
impl Shell {
    pub(super) fn new(interactive: bool) -> Self {
        Shell {
            last_status: 0,
            interactive,
            should_exit: false,
            exit_code: 0,
            variables: BTreeMap::new(),
        }
    }

    pub(super) fn set_var(&mut self, name: &[u8], value: &[u8]) {
        self.variables.insert(name.to_vec(), value.to_vec());
    }

    pub(super) fn get_var(&self, name: &[u8]) -> Option<&[u8]> {
        self.variables.get(name).map(|v| v.as_slice())
    }
}
