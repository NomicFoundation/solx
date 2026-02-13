//!
//! `solx` executable and `solc` client library.
//!

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::result_large_err)]

#[cfg(feature = "solc")]
pub(crate) mod solc;

#[cfg(feature = "solc")]
pub use self::solc::Solc;
