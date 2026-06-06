//!
//! `try` statement lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::CatchClause;
use slang_solidity_v2::ast::TryStatement;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits a `try` statement (recognise the external call, `sol.if(status)`,
    /// success body, failure selector dispatch `Error`/`Panic`, fallback).
    pub fn emit_try(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let _ = (try_statement, block);
        unimplemented!("try statement")
    }

    /// Binds the declared `returns (...)` of a `try` from the decoded results.
    pub fn bind_try_returns(
        &mut self,
        try_statement: &TryStatement,
        results: &[Value<'context, 'block>],
        then_entry: &BlockRef<'context, 'block>,
    ) {
        let _ = (try_statement, results, then_entry);
        unimplemented!("try return binding")
    }

    /// Decodes the returndata from `start` into `result_types`
    /// (`GetReturnData` + `Decode`).
    pub fn emit_returndata_decode(
        &self,
        start: i64,
        result_types: &[Type<'context>],
        block: &BlockRef<'context, 'block>,
    ) -> Vec<Value<'context, 'block>> {
        let _ = (start, result_types, block);
        unimplemented!("returndata decode")
    }

    /// Emits a typed `catch Error(...)` / `catch Panic(...)` clause (bind the
    /// param past the 4-byte selector, body, yield).
    pub fn emit_typed_catch_clause(
        &mut self,
        clause: &CatchClause,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let _ = (clause, block);
        unimplemented!("typed catch clause")
    }

    /// Emits the fallback `catch (...)` / `catch {}` clause; `None` ⇒ re-revert
    /// the exact returndata + dead yield.
    pub fn emit_fallback_catch_clause(
        &mut self,
        clause: Option<&CatchClause>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let _ = (clause, block);
        unimplemented!("fallback catch clause")
    }

    /// The non-try-lowerable fallback: emit the call, bind the first return,
    /// then the body.
    pub fn emit_try_success_only(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let _ = (try_statement, block);
        unimplemented!("try success-only")
    }
}
