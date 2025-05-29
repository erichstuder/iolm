#![cfg_attr(not(test), no_std)]

#[cfg(all(feature = "defmt", feature = "log"))]
compile_error!("Features 'defmt' and 'log' cannot be enabled at the same time.");

#[cfg(feature = "master")]
pub mod master_dl;
#[cfg(feature = "device")]
pub mod device_dl;
