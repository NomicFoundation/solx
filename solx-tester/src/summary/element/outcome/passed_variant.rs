//!
//! `solx` tester summary element passed outcome variant.
//!

///
/// `solx` tester summary element passed outcome variant.
///
#[derive(Debug)]
pub enum PassedVariant {
    /// Deploy transaction.
    Deploy {
        /// Deploy code size.
        deploy_size: u64,
        /// Runtime code size.
        runtime_size: u64,
        /// Amount of gas used.
        gas: u64,
    },
    /// Runtime transaction.
    Runtime {
        /// Amount of gas used.
        gas: u64,
    },
    /// A special function call.
    Special,
}
