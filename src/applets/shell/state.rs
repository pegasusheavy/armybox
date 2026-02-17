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
    /// Shell aliases (name -> value)
    pub(super) aliases: BTreeMap<Vec<u8>, Vec<u8>>,
    /// Positional parameters ($0, $1, $2, ...)
    /// positional_params[0] = $0 (script name / shell name)
    /// positional_params[1] = $1, etc.
    pub(super) positional_params: Vec<Vec<u8>>,
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
            aliases: BTreeMap::new(),
            positional_params: Vec::new(),
        }
    }

    pub(super) fn set_var(&mut self, name: &[u8], value: &[u8]) {
        self.variables.insert(name.to_vec(), value.to_vec());
    }

    pub(super) fn get_var(&self, name: &[u8]) -> Option<&[u8]> {
        self.variables.get(name).map(|v| v.as_slice())
    }

    /// Get a positional parameter by index (0 = $0, 1 = $1, etc.)
    pub(super) fn get_positional(&self, index: usize) -> Option<&[u8]> {
        self.positional_params.get(index).map(|v| v.as_slice())
    }

    /// Set positional parameters from script arguments.
    /// `script_name` becomes $0, `args` become $1, $2, ...
    pub(super) fn set_positional_params(&mut self, script_name: &[u8], args: &[&[u8]]) {
        self.positional_params.clear();
        self.positional_params.push(script_name.to_vec());
        for arg in args {
            self.positional_params.push(arg.to_vec());
        }
    }

    /// Number of positional parameters ($# — does not count $0)
    pub(super) fn param_count(&self) -> usize {
        if self.positional_params.is_empty() {
            0
        } else {
            self.positional_params.len() - 1
        }
    }
}
