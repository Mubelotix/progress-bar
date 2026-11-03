#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod pb;
pub mod style;
pub mod output;
pub mod clock;
mod eta;

#[cfg(feature = "std")]
pub mod global;

#[cfg(feature = "logger")]
pub(crate) mod logger;

pub use style::*;
pub use output::*;
pub use clock::*;

#[cfg(feature = "std")]
pub use global::*;

#[cfg(feature = "logger")]
pub use logger::*;
