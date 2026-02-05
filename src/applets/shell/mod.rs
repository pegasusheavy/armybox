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

mod arithmetic;
mod builtins;
mod control;
mod entry;
mod execute;
mod expand;
mod parser;
mod state;
mod util;

// Re-export get_arg from parent for entry.rs
pub(self) use super::get_arg;

// Re-export the public entry points
pub use entry::{ash, dash, sh};

// Re-export items needed by submodules via super::
// These allow submodules to use `super::function_name` instead of fully qualified paths

#[cfg(feature = "alloc")]
pub(self) use execute::execute_script;

#[cfg(feature = "alloc")]
pub(self) use expand::expand_string;

#[cfg(feature = "alloc")]
pub(self) use parser::parse_word;

#[cfg(feature = "alloc")]
pub(self) use util::skip_whitespace_and_comments;
