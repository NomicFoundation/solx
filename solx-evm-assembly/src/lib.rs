//!
//! EVM assembly translator.
//!

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::result_large_err)]

pub mod assembly;
pub mod ethereal_ir;
pub mod extra_metadata;

pub use self::assembly::Assembly;
pub use self::extra_metadata::defined_function::DefinedFunction as ExtraMetadataRecursiveFunction;
pub use self::extra_metadata::ExtraMetadata;
