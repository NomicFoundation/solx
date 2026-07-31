//!
//! Signature parameter for the Sol function reference types.
//!

use melior::ir::Type as MlirType;
use melior::ir::TypeLike;
use melior::ir::r#type::FunctionType as MlirFunctionType;

use crate::Type;
use crate::ffi;

/// The types a function is called through: what a `sol::FuncRefType` is parameterized by, and what
/// a call names as its callee type. The zero-argument default is the unit signature `() -> ()`.
#[derive(Default, Clone)]
pub struct FunctionType<'context> {
    /// Parameter types, in declaration order.
    pub parameters: Vec<Type<'context>>,
    /// Result types, one per returned value: the dialect has no tuple.
    pub results: Vec<Type<'context>>,
}

impl<'context> FunctionType<'context> {
    /// The melior function type this signature interns to: what a `sol.func` declares and what a
    /// function reference is parameterized by.
    pub fn to_mlir(&self, context: &'context melior::Context) -> MlirFunctionType<'context> {
        let parameters: Vec<MlirType<'context>> = self
            .parameters
            .iter()
            .map(|parameter| parameter.into_mlir())
            .collect();
        let results: Vec<MlirType<'context>> = self
            .results
            .iter()
            .map(|result| result.into_mlir())
            .collect();
        MlirFunctionType::new(context, &parameters, &results)
    }

    /// A `sol::FuncRefType` over this signature, an internal function pointer.
    pub fn reference(&self, context: &'context melior::Context) -> Type<'context> {
        Type::new(unsafe {
            MlirType::from_raw(ffi::solxCreateFuncRefType(
                context.to_raw(),
                MlirType::from(self.to_mlir(context)).to_raw(),
            ))
        })
    }
}
