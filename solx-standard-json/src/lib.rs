//!
//! `solx` standard JSON protocol.
//!

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::result_large_err)]

pub mod input;
pub mod output;
pub mod version;

pub use self::input::Input;
pub use self::input::language::Language as InputLanguage;
pub use self::input::settings::Settings as InputSettings;
pub use self::input::settings::debug::Debug as InputDebug;
pub use self::input::settings::metadata::Metadata as InputMetadata;
pub use self::input::settings::optimizer::Optimizer as InputOptimizer;
pub use self::input::settings::selection::Selection as InputSelection;
pub use self::input::settings::selection::selector::Selector as InputSelector;
pub use self::input::source::Source as InputSource;
pub use self::output::Output;
pub use self::output::contract::Contract as OutputContract;
pub use self::output::contract::evm::EVM as OutputContractEVM;
pub use self::output::contract::evm::bytecode::Bytecode as OutputContractEVMBytecode;
pub use self::output::error::Error as OutputError;
pub use self::output::error::collectable::Collectable as CollectableError;
pub use self::output::error::source_location::SourceLocation as OutputErrorSourceLocation;
pub use self::output::source::Source as OutputSource;
pub use self::version::Version;

/// The standard JSON debug environment variable.
pub const STANDARD_JSON_DEBUG_ENV: &str = "SOLX_STANDARD_JSON_DEBUG";
