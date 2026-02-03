//!
//! The Yul source code type.
//!

use crate::declare_wrapper;

declare_wrapper!(solx_yul::YulType, Type);

///
/// The Yul source code type.
///
impl Type {
    ///
    /// Converts the type into its LLVM.
    ///
    pub fn into_llvm<'ctx, C>(self, context: &C) -> inkwell::types::IntType<'ctx>
    where
        C: solx_codegen_evm::IContext<'ctx>,
    {
        match self.0 {
            solx_yul::YulType::Bool => context.integer_type(solx_utils::BIT_LENGTH_BOOLEAN),
            solx_yul::YulType::Int(bitlength) => context.integer_type(bitlength),
            solx_yul::YulType::UInt(bitlength) => context.integer_type(bitlength),
            solx_yul::YulType::Custom(_) => context.field_type(),
        }
    }
}
