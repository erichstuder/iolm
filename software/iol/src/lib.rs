//! # iol
//!
//! ## Features
//!
//! - `master`: Builds for an IO-Link Master.
//! - `iols`: Enables IO-Link Safety features.
//! - `defmt`: Enables defmt logging (for embedded).
//! - `log`: Enables standard log crate logging.
//!
//! **Note:** `defmt` and `log` cannot be enabled at the same time.

#![cfg_attr(not(test), no_std)]

#[cfg(all(feature = "defmt", feature = "log"))]
compile_error!("Features 'defmt' and 'log' cannot be enabled at the same time.");

#[cfg(feature = "master")]
pub mod master;
mod common;

// #[cfg(feature = "device")]
// pub mod device;
