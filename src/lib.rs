#![doc = include_str!("../README.md")]

//! # Platform dispatch
//!
//! [`OsLogLayer`] is a different concrete type on each platform,
//! selected at compile time. The public API is identical:
//!
//! ```
//! use tracing_os_layer::OsLogLayer;
//!
//! let _layer = OsLogLayer::try_new("com.example.app", "worker");
//! ```

#[cfg(target_vendor = "apple")]
mod macos;
#[cfg(target_vendor = "apple")]
pub use macos::OsLogLayer;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::OsLogLayer;

#[cfg(not(any(target_vendor = "apple", target_os = "linux")))]
mod stub;
#[cfg(not(any(target_vendor = "apple", target_os = "linux")))]
pub use stub::OsLogLayer;
