//!
//! The deployable-object emission trait: a contract or library lowers itself to
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

/// Lowers a top-level definition to one deployable `sol.contract` object,
/// threading `&mut Context` (object emission mutates the redirect / binding /
/// current-contract state). Implemented by `ContractDefinition` and
/// `LibraryDefinition`: a library is a degenerate contract whose `emit` omits
/// the constructor / state / inheritance steps. The shared `sol.contract` shell
/// and signature pre-registration are provided methods both share.
pub trait EmitObject {
    /// Emits this definition as a deployable `sol.contract` with its functions.
    /// `context` is the per-object MLIR builder state; `scope` carries the unit
    /// function inputs (kept off `Context` so the Slang AST stays off the builder).
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
        let builder = &context.builder;
        mlir_region_op!(
            builder,
            block,
            ContractOperation
                .sym_name(StringAttribute::new(builder.context, name))
                .kind(kind.attribute(builder.context))
            ; body_region
        )
    }

    /// Resolves and registers each `(function, symbol)`'s MLIR signature into the
    /// context so a call resolves before any body is emitted. The caller supplies
    /// the symbol per registration site — canonical, node-id-qualified for a free
    /// function, or contract-qualified for a shadowed base override.
    fn register_signatures(
        &self,
        context: &mut Context,
        functions: impl IntoIterator<Item = (FunctionDefinition, String)>,
    ) {
        for (function, symbol) in functions {
            let (parameter_types, return_types) = Type::resolve_signature(
                &function,
                LocationPolicy::Declared(None),
                &context.builder,
            );
            context.register_function_signature(
                function.node_id(),
                symbol,
                parameter_types,
                return_types,
            );
        }
    }
}
