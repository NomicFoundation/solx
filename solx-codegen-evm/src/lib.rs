//!
//! EVM codegen library.
//!

#![allow(clippy::upper_case_acronyms)]

pub(crate) mod attribute;
pub(crate) mod codegen;
pub(crate) mod r#const;
pub(crate) mod debug_config;
pub(crate) mod optimizer;
pub(crate) mod target_machine;

pub use self::codegen::IS_SIZE_FALLBACK;
pub use self::codegen::append_metadata;
pub use self::codegen::assemble;
pub use self::codegen::build::Build;
pub use self::codegen::context::Context;
pub use self::codegen::initialize_target;
pub use self::codegen::link;
pub use self::codegen::minimal_deploy_code;
pub use self::r#const::*;
pub use self::debug_config::OutputConfig;
pub use self::optimizer::Optimizer;
pub use self::optimizer::settings::Settings as OptimizerSettings;
