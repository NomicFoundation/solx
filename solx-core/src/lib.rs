//!
//! Solidity compiler library.
//!

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::result_large_err)]
#![allow(clippy::large_enum_variant)]

pub mod arguments;
pub mod build;
pub mod compiler;
pub mod r#const;
pub mod error;
pub mod frontend;
pub mod process;
pub mod project;

pub use self::arguments::Arguments;
pub use self::build::Build as EVMBuild;
pub use self::build::contract::Contract as EVMContractBuild;
pub use self::compiler::Compiler;
pub use self::r#const::*;
pub use self::error::Error;
pub use self::error::stack_too_deep::StackTooDeep as StackTooDeepError;
pub use self::frontend::Frontend;
pub use self::process::EXECUTABLE;
pub use self::process::child::run as run_subprocess;
pub use self::process::job::Job as EVMProcessJob;
pub use self::process::output::Output as EVMProcessOutput;
pub use self::process::pool::Pool as EVMProcessPool;
pub use self::process::session::Session as EVMProcessSession;
pub use self::project::Project;
pub use self::project::contract::Contract as ProjectContract;

/// The default error compatible with `solc` standard JSON output.
pub type Result<T> = std::result::Result<T, Error>;
