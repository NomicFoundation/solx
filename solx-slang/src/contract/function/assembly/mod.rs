//!
//! Inline assembly lowering: the `sol.inline_asm` region and the Yul functions it declares.
//!

pub mod expression;
pub mod path;
pub mod reference;
pub mod statement;

use slang_solidity_v2::ast::AssemblyStatement;
use slang_solidity_v2::ast::YulFunctionDefinition;

use solx_mlir::Pointer;
use solx_mlir::Word;
use solx_mlir::YulFunction;

use crate::scope::assembly::AssemblyScope;
use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// An `assembly { .. }` statement: a `sol.inline_asm` region holding the block's Yul dialect
    /// ops.
    pub fn assembly_statement(&mut self, node: &AssemblyStatement) {
        let body = self.current_block().inline_asm(node.is_memory_safe(), self);
        self.assembly(body, |scope| scope.statements(&node.body()));
    }
}

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// Emits `definition`'s `yul.func` at its first naming, whether that is a call site or the
    /// definition statement, marking it before the body so a recursive call resolves, and hands
    /// back the signature a call names it by. The ops are inserted ahead of the body's own
    /// statements in first-naming order, so a function named from an earlier function's body
    /// precedes one whose definition comes first in source.
    ///
    /// Parameters and return variables are pointers like any Yul variable: the parameter takes the
    /// incoming block argument, the return variable the zero a Yul return defaults to.
    pub fn function_definition(
        &mut self,
        definition: &YulFunctionDefinition,
    ) -> YulFunction<'context> {
        if let Some(signature) = self.function_signatures.get(&definition.node_id()) {
            return signature.clone();
        }
        let position = self.function_signatures.len();
        let signature = self.signature(definition);
        let entry = signature.define(position, self, self.body);

        self.function_body(entry, |scope| {
            for (index, parameter) in definition.parameters().iter().enumerate() {
                let argument = entry.argument(index);
                scope.bind(parameter.node_id(), argument);
            }
            let returns: Vec<Pointer> = match definition.returns() {
                Some(names) => names
                    .iter()
                    .map(|name| {
                        let zero = Word::zero(scope);
                        scope.bind(name.node_id(), zero)
                    })
                    .collect(),
                None => Vec::new(),
            };

            scope.in_function(returns, |scope| {
                scope.statements(&definition.body());
                if !scope.current_block().is_terminated() {
                    scope.function_return();
                }
            });
        });
        signature
    }

    /// Emits `yul.func_return` carrying the current words of the enclosing Yul function's return
    /// variables.
    pub fn function_return(&self) {
        let operands: Vec<_> = self
            .returns
            .iter()
            .map(|pointer| pointer.load(self))
            .collect();
        self.current_block().function_return(&operands, self);
    }
}
