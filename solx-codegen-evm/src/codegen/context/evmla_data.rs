//!
//! The LLVM IR generator EVM legacy assembly data.
//!

use crate::context::traits::evmla_data::IEVMLAData;
use crate::context::value::Value;

///
/// The LLVM IR generator EVM legacy assembly data.
///
/// Describes some data that is only relevant to the EVM legacy assembly.
///
#[derive(Debug, Clone)]
pub struct EVMLAData<'ctx> {
    /// The Solidity compiler version.
    /// Some instruction behave differenly depending on the version.
    pub version: semver::Version,
    /// The static stack allocated for the current function.
    pub stack: Vec<Value<'ctx>>,
}

impl EVMLAData<'_> {
    /// The default stack size.
    pub const DEFAULT_STACK_SIZE: usize = 64;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(version: semver::Version) -> Self {
        Self {
            version,
            stack: Vec::with_capacity(Self::DEFAULT_STACK_SIZE),
        }
    }
}

impl<'ctx> IEVMLAData<'ctx> for EVMLAData<'ctx> {
    fn get_element(&self, position: usize) -> &Value<'ctx> {
        &self.stack[position]
    }

    fn set_element(&mut self, position: usize, value: Value<'ctx>) {
        self.stack[position] = value;
    }

    fn set_original(&mut self, position: usize, original: String) {
        self.stack[position].original = Some(original);
    }
}
