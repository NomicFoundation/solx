//!
//! Signature parameter for the Sol function reference types.
//!

use melior::ir::Type as MlirType;
use melior::ir::TypeLike;

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
    /// Records a function's MLIR-interned parameter and result types.
    pub fn new(parameters: Vec<Type<'context>>, results: Vec<Type<'context>>) -> Self {
        Self {
            parameters,
            results,
        }
    }

    /// A `sol::FuncRefType` over this signature, an internal function pointer.
    pub fn reference(&self, context: &'context melior::Context) -> Type<'context> {
        let parameters: Vec<mlir_sys::MlirType> = self
            .parameters
            .iter()
            .map(|parameter| parameter.inner.to_raw())
            .collect();
        let results: Vec<mlir_sys::MlirType> = self
            .results
            .iter()
            .map(|result| result.inner.to_raw())
            .collect();
        Type::new(unsafe {
            MlirType::from_raw(ffi::solxCreateFuncRefType(
                context.to_raw(),
                parameters.as_ptr(),
                parameters.len(),
                results.as_ptr(),
                results.len(),
            ))
        })
    }
}
