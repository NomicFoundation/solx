//!
//! The persistent worker subprocess pool and its framed session/job protocol.
//!

pub mod channel;
pub mod child;
pub mod job;
pub mod output;
pub mod pool;
pub mod session;
pub mod worker;

use std::path::PathBuf;
use std::sync::OnceLock;

/// The overridden executable name used when the compiler is run as a library.
pub static EXECUTABLE: OnceLock<PathBuf> = OnceLock::new();
