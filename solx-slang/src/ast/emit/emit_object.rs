//!
//! The deployable-object emission trait: a contract or library emits itself as
//! a `sol.contract`.
//!

use melior::ir::BlockRef;
use melior::ir::attribute::StringAttribute;

use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Context;
use solx_mlir::ContractKind;
use solx_mlir::LocationPolicy;
use solx_mlir::Type;
use solx_mlir::ods::sol::ContractOperation;

use crate::ast::contract::ObjectScope;

/// Emits a top-level definition as one deployable `sol.contract`, threading `&mut Context`.
/// Implemented by `ContractDefinition` and `LibraryDefinition`, where a library omits the constructor and state steps.
pub trait EmitObject {
    /// Emits this definition as a deployable `sol.contract` with its functions.
    fn emit(&self, context: &mut Context, scope: &ObjectScope);

    /// Emits the empty `sol.contract` shell, returning its body block for
    /// appending the object's function definitions. The kind builds its own
    /// dialect attribute.
    fn emit_contract_shell<'context, 'block>(
        &self,
        context: &Context<'context>,
        name: &str,
        kind: ContractKind,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        mlir_region_op!(
            context,
            block,
            ContractOperation
                .sym_name(StringAttribute::new(context.mlir_context, name))
                .kind(kind.attribute(context.mlir_context))
            ; body_region
        )
    }

    /// Resolves and registers each `(function, symbol)`'s MLIR signature so a call resolves before
    /// any body is emitted (the caller supplies the symbol per site).
    fn register_signatures(
        &self,
        context: &mut Context,
        functions: impl IntoIterator<Item = (FunctionDefinition, String)>,
    ) {
        for (function, symbol) in functions {
            let (parameter_types, return_types) =
                Type::resolve_signature(&function, LocationPolicy::Declared(None), context);
            context.register_function_signature(
                function.node_id(),
                symbol,
                parameter_types,
                return_types,
            );
        }
    }
}
