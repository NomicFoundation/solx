//!
//! The LLVM IR EVMLA data trait.
//!

use crate::context::value::Value;

///
/// The LLVM IR EVMLA data trait.
///
pub trait IEVMLAData<'ctx> {
    ///
    /// Returns the element from the specified stack position.
    ///
    /// # Panics
    /// If `position` is out of bounds.
    ///
    fn get_element(&self, position: usize) -> &Value<'ctx>;
}
