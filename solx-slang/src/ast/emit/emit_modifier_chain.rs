//!
//! The modifier-chain emission trait: a modifier-wrapped function lowers its
//! modifier stages to the internal `sol.func` chain (`$mod0 … $modN`, `$body`).
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;

use slang_solidity_v2::ast::Block;

use solx_mlir::Environment;

use crate::ast::contract::function::FunctionScope;
use crate::ast::contract::function::modifier::ModifiedBody;
use crate::ast::contract::function::modifier::ModifierStageParams;

/// Lowers a function's modifier invocations to the modifier-stage `sol.func` chain, consumed by
/// function emission (a modified body) and constructor emission (a constructor's modifiers).
pub trait EmitModifierChain {
    /// Emits a modifier-wrapped function as a chain of internal `sol.func`s (each stage its own func,
    /// so a `return` in a modifier exits only that stage). Returns the entry's fall-through block.
    fn emit_modified_body<'state, 'context, 'frame, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        frame: &ModifiedBody<'frame, 'context, 'block>,
        environment: &mut Environment<'context, 'block>,
        return_slots: &mut Vec<Option<Value<'context, 'block>>>,
        modifier_stages: Vec<Block>,
        modifier_stage_params: Vec<ModifierStageParams<'context, 'block>>,
        current_block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>>;

    /// Emits one modifier stage as an internal `sol.func`, parameterised by
    /// `[this modifier's arguments ++ downstream values ++ threaded return
    /// values]`, whose `_;` calls `next_symbol`.
    fn emit_modifier_stage_func<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        stage_symbol: &str,
        modifier_body: &Block,
        modifier_params: &ModifierStageParams<'context, '_>,
        downstream_types: &[Type<'context>],
        result_types: &[Type<'context>],
        next_symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    );

    /// Resolves the function's modifier invocations to their bodies, evaluating
    /// each invocation's arguments in `environment` into fresh per-invocation
    /// allocas. Returns, outermost first, the stage statements, the parallel
    /// per-stage parameter bindings, and the block after the argument evaluations.
    fn build_modifier_stages<'state, 'context, 'env>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        environment: &Environment<'context, 'env>,
        block: BlockRef<'context, 'env>,
    ) -> (
        Vec<Block>,
        Vec<ModifierStageParams<'context, 'env>>,
        BlockRef<'context, 'env>,
    );
}
