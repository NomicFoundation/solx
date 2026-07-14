//!
//! Function parameter projection: the MLIR type a declared parameter resolves to.
//!

use slang_solidity_v2::ast::Parameter as SlangParameter;

use solx_mlir::Context as MlirContext;
use solx_mlir::Type as MlirType;

codegen!(
    Parameter {
        /// The MLIR type of the declared parameter.
        pub fn resolve<'context>(
            parameter: &SlangParameter,
            context: &MlirContext<'context>,
        ) -> MlirType<'context> {
            codegen!(@result_type Parameter, parameter, context)
        }
    }
);
