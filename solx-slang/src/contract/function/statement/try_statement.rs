//!
//! The `try`/`catch` statement guarding an external call or a contract creation.
//!

use slang_solidity_v2::ast::CatchClause;
use slang_solidity_v2::ast::CatchClauseKind;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TryStatement;

use solx_mlir::FallbackRegion;

use crate::contract::function::expression::call::Call;
use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `try`/`catch` statement: the guarded call's status branches into one region per declared
    /// clause, the returns binding the call's results in the success region.
    pub fn try_statement(&mut self, node: &TryStatement) {
        let Expression::FunctionCallExpression(call) = node.expression() else {
            unimplemented!("a `try` statement guards a call");
        };
        let (status, values) = Call::try_call(&call, self);

        let (mut panic, mut error, mut fallback) = (None, None, None);
        for clause in node.catch_clauses().iter() {
            match clause.kind().expect("slang classifies every catch clause") {
                CatchClauseKind::Panic => panic = Some(clause),
                CatchClauseKind::Error => error = Some(clause),
                CatchClauseKind::LowLevel => fallback = Some(clause),
            }
        }
        let regions = self.current_block().r#try(
            status,
            panic.is_some(),
            error.is_some(),
            fallback.as_ref().map(|clause| match clause.error() {
                Some(_) => FallbackRegion::ReturnData,
                None => FallbackRegion::Unbound,
            }),
            self,
        );

        self.nested(|scope| {
            scope.region(regions.success, |scope| {
                if let Some(returns) = node.returns() {
                    for (parameter, value) in returns.iter().zip(values) {
                        let Some(name) = parameter.name() else {
                            continue;
                        };
                        let declared_type = scope.typing(parameter.get_type());
                        scope.define_local(name.name(), declared_type, |_scope| value);
                    }
                }
                scope.block(&node.body());
            });
        });
        for (block, clause) in [
            regions.panic.zip(panic),
            regions.error.zip(error),
            regions.fallback.zip(fallback),
        ]
        .into_iter()
        .flatten()
        {
            self.nested(|scope| {
                scope.region(block, |scope| scope.catch_clause(&clause));
            });
        }
    }

    /// A catch clause's body, its declared parameter bound from the region's block argument.
    fn catch_clause(&mut self, node: &CatchClause) {
        if let Some(error) = node.error()
            && let Some(parameter) = error.parameters().iter().next()
            && let Some(name) = parameter.name()
        {
            let declared_type = self.typing(parameter.get_type());
            let bound = self.current_block().argument(0);
            self.define_local(name.name(), declared_type, |_scope| bound);
        }
        self.block(&node.body());
    }
}
