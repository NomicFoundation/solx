//!
//! Inline-assembly (Yul) statement lowering.
//!

/// Yul EVM-opcode intrinsic lowering.
pub mod intrinsic;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::AssemblyStatement;
use slang_solidity_v2::ast::YulBlock;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulPath;
use slang_solidity_v2::ast::YulStatement;
use slang_solidity_v2::ast::YulValueCase;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits an `assembly { … }` block (pre-register Yul function definitions,
    /// emit each non-definition statement, remove the added entries).
    pub fn emit_assembly(
        &mut self,
        assembly: &AssemblyStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let _ = (assembly, block);
        unimplemented!("assembly block")
    }

    /// Emits one Yul statement — an exhaustive `match` over all 11
    /// [`YulStatement`] variants (no `_`). Returns `BlockRef` (NOT `Option`):
    /// Yul never terminates solx control flow.
    pub fn emit_yul_statement(
        &mut self,
        statement: &YulStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let _ = block;
        match statement {
            YulStatement::YulBlock(_) => unimplemented!("yul: block"),
            YulStatement::YulFunctionDefinition(_) => unimplemented!("yul: function definition"),
            YulStatement::YulIfStatement(_) => unimplemented!("yul: if"),
            YulStatement::YulForStatement(_) => unimplemented!("yul: for"),
            YulStatement::YulSwitchStatement(_) => unimplemented!("yul: switch"),
            YulStatement::YulLeaveStatement(_) => unimplemented!("yul: leave"),
            YulStatement::YulBreakStatement(_) => unimplemented!("yul: break"),
            YulStatement::YulContinueStatement(_) => unimplemented!("yul: continue"),
            YulStatement::YulVariableAssignmentStatement(_) => unimplemented!("yul: assignment"),
            YulStatement::YulVariableDeclarationStatement(_) => unimplemented!("yul: declaration"),
            YulStatement::YulExpression(_) => unimplemented!("yul: expression statement"),
        }
    }

    /// Emits a Yul region's statements (fold, stop after a terminator,
    /// `sol.yield`).
    pub fn emit_yul_region_statements(
        &mut self,
        body: &YulBlock,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let _ = (body, block);
        unimplemented!("yul region statements")
    }

    /// Emits a Yul `switch` as a nested `sol.if` chain keyed by `sol.cmp Eq`,
    /// with the default body in the deepest else.
    ///
    /// R8-8: `block: &mut BlockRef` out-param (the oracle mutates the parent
    /// block in place rather than threading by value) — kept; flagged.
    pub fn emit_yul_switch_chain(
        &mut self,
        selector: Value<'context, 'block>,
        value_cases: &[YulValueCase],
        default_body: Option<&YulBlock>,
        block: &mut BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let _ = (selector, value_cases, default_body, block);
        unimplemented!("yul switch chain")
    }

    /// Emits one Yul expression — an exhaustive 3-arm `match` over
    /// [`YulExpression`] (Literal / Path / Call).
    pub fn emit_yul_expression(
        &mut self,
        expression: &YulExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = block;
        match expression {
            YulExpression::YulLiteral(_) => unimplemented!("yul: literal"),
            YulExpression::YulPath(_) => unimplemented!("yul: path"),
            YulExpression::YulFunctionCallExpression(_) => unimplemented!("yul: call"),
        }
    }

    /// Emits a Yul path read (`Constant` / `stateVar.slot` / `.offset` /
    /// local-var load + widen). The `.slot`/`.offset` dispatch keys the typed
    /// `BuiltIn::Yul*` suffix (R8-9), NOT `name == "slot"/"offset"`.
    pub fn emit_yul_path(
        &self,
        path: &YulPath,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = (path, block);
        unimplemented!("yul path")
    }

    /// Whether a Yul statement terminates its region (`break` / `continue`);
    /// `yul.return` / `yul.revert` are effects, not terminators.
    pub fn is_terminating_yul_statement(statement: &YulStatement) -> bool {
        matches!(
            statement,
            YulStatement::YulBreakStatement(_) | YulStatement::YulContinueStatement(_)
        )
    }

    /// Emits a call of a user-defined Yul function (single result); the
    /// recursion guard `if *depth >= 1 { unimplemented!(...) }` stays.
    pub fn emit_yul_user_call(
        &mut self,
        name: &str,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = (name, arguments, block);
        unimplemented!("yul user call")
    }

    /// Emits a call of a user-defined Yul function (multiple results); the
    /// recursion guard stays.
    pub fn emit_yul_user_call_multi(
        &mut self,
        name: &str,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let _ = (name, arguments, block);
        unimplemented!("yul user call (multi)")
    }
}
