//! POSIX Shell implementation
//!
//! A minimal but functional shell supporting:
//! - Command execution
//! - Pipes (cmd1 | cmd2)
//! - Redirections (>, >>, <, 2>)
//! - Environment variables ($VAR)
//! - Variable assignment (VAR=value)
//! - Control structures (if/then/else/fi, for/do/done, while/do/done, case/esac)
//! - Command substitution $(cmd)
//! - Arithmetic expansion $((expr))
//! - Built-in commands (cd, exit, export, etc.)
//! - Script execution
//!
//! ## Module Organization
//!
//! The shell implementation is split into the following submodules:
//! - `state` - Shell state management (variables, exit status)
//! - `entry` - Entry points (sh, ash, dash) and interactive loop
//! - `parser` - Tokenization and command parsing
//! - `execute` - Script and pipeline execution
//! - `expand` - Variable and command substitution
//! - `arithmetic` - Arithmetic expression evaluation
//! - `control` - Control flow structures (if, while, for, case)
//! - `builtins` - Built-in commands
//! - `util` - Utility functions

#[cfg(feature = "sh")]
mod arithmetic;
#[cfg(feature = "sh")]
mod builtins;
#[cfg(feature = "sh")]
mod control;
#[cfg(feature = "sh")]
mod entry;
#[cfg(feature = "sh")]
mod execute;
#[cfg(feature = "sh")]
mod expand;
#[cfg(feature = "sh")]
mod parser;
#[cfg(feature = "sh")]
mod state;
#[cfg(feature = "sh")]
mod util;

// Re-export get_arg from parent for entry.rs
#[cfg(feature = "sh")]
pub(self) use super::get_arg;

// Re-export the public entry points
#[cfg(feature = "sh")]
pub use entry::{ash, dash, sh};

// Re-export items needed by submodules via super::
// These allow submodules to use `super::function_name` instead of fully qualified paths

#[cfg(all(feature = "sh", feature = "alloc"))]
pub(self) use execute::execute_script;

#[cfg(all(feature = "sh", feature = "alloc"))]
pub(self) use expand::expand_string;

#[cfg(all(feature = "sh", feature = "alloc"))]
pub(self) use parser::parse_word;

#[cfg(all(feature = "sh", feature = "alloc"))]
pub(self) use util::skip_whitespace_and_comments;
