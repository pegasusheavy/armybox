//! Pure Rust no_std Zstandard (zstd) compression and decompression
//!
//! Implements RFC 8878 with no external dependencies beyond alloc.

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod common;
pub mod bitstream;
pub mod fse;
pub mod huff;
pub mod decompress;
pub mod compress;
pub mod xxhash;

pub use common::ZstdError;
pub use decompress::{decompress, decompress_bounded};
pub use compress::compress;
