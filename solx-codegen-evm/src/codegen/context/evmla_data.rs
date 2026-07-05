//!
//! The LLVM IR generator EVM legacy assembly data.
//!

use crate::context::traits::evmla_data::IEVMLAData;
use crate::context::traits::evmla_data::shadow_slot::ShadowSlot;
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
    /// The block-local resolution of each stack position onto its live value.
    pub shadow: Vec<ShadowSlot<'ctx>>,
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
            shadow: Vec::with_capacity(Self::DEFAULT_STACK_SIZE),
        }
    }
}

impl<'ctx> IEVMLAData<'ctx> for EVMLAData<'ctx> {
    fn get_element(&self, position: usize) -> &Value<'ctx> {
        &self.stack[position]
    }

    fn shadow_reset(&mut self) {
        self.shadow.clear();
        self.shadow
            .extend((0..self.stack.len()).map(ShadowSlot::Memory));
    }

    fn shadow_peek(&self, position: usize) -> ShadowSlot<'ctx> {
        self.shadow[position]
    }

    fn shadow_write(&mut self, position: usize, value: inkwell::values::BasicValueEnum<'ctx>) {
        self.shadow[position] = ShadowSlot::Value(value);
    }

    fn shadow_dup(&mut self, destination: usize, source: usize) {
        self.shadow[destination] = self.shadow[source];
    }

    fn shadow_swap(&mut self, first: usize, second: usize) {
        self.shadow.swap(first, second);
    }
}
