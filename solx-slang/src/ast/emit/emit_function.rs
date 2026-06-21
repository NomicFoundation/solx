//!
//! The function-definition emission trait: a function lowers itself to a
//! `sol.func`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;

use solx_mlir::Environment;

use crate::ast::contract::function::FunctionScope;
use crate::ast::contract::function::body_kind::BodyKind;

/// Lowers a function definition to a `sol.func`. A contract / library threads the
/// emission via the shared [`FunctionScope`]; the node carries the projection.
pub trait EmitFunction {
    /// Emits a `sol.func` under the function's canonical (dispatchable) symbol.
    fn emit<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    );

    /// Emits the function under an explicit `symbol` with no public selector —
    /// for free and library functions, resolved by node id, never dispatched.
    fn emit_with_symbol<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    );

    /// Opens the `sol.func`, binds parameters and return slots, threads the body
    /// statements (or the modifier chain), and closes with the default return.
    /// `symbol_override` names the `sol.func` explicitly and suppresses the public
    /// selector and special kind.
    fn emit_inner<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
        body_kind: BodyKind,
    );

    /// Allocates a stack slot for each parameter, stores the incoming argument
    /// into it, and binds the slot to the parameter name in `environment`.
    fn bind_parameters<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        parameter_types: &[Type<'context>],
        entry_block: &BlockRef<'context, 'block>,
        environment: &mut Environment<'context, 'block>,
    );

    /// Allocates and binds a stack slot for each named return value, pushing
    /// `None` for an unnamed return. A modifier body seeds every slot from the
    /// trailing block arguments instead.
    fn initialize_return_slots<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        result_types: &[Type<'context>],
        parameter_count: usize,
        body_kind: BodyKind,
        entry_block: &BlockRef<'context, 'block>,
        environment: &mut Environment<'context, 'block>,
    ) -> Vec<Option<Value<'context, 'block>>>;

    /// Emits a default `sol.return` when the block lacks a terminator, loading
    /// named-return slots and materialising a typed default for the rest.
    fn emit_default_return<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        result_types: &[Type<'context>],
        return_slots: &[Option<Value<'context, 'block>>],
        block: &BlockRef<'context, 'block>,
    );
}
