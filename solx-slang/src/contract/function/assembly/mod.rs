//!
//! Inline assembly lowering: the `sol.inline_asm` region and the Yul functions it declares.
//!

pub mod expression;
pub mod reference;
pub mod statement;

use slang_solidity_v2::ast::AssemblyStatement;
use slang_solidity_v2::ast::YulBlock;
use slang_solidity_v2::ast::YulFunctionDefinition;
use slang_solidity_v2::ast::YulStatement;

use solx_mlir::Slot;
use solx_mlir::YulFunction;

use crate::scope::assembly::AssemblyScope;
use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// An `assembly { .. }` statement: a `sol.inline_asm` region holding the block's Yul dialect
    /// ops. The `memory-safe` flag rides on the op so the memory model of the enclosing code is not
    /// invalidated by a block that promises not to touch memory Solidity owns.
    pub fn assembly_statement(&mut self, node: &AssemblyStatement) {
        let block = self
            .current_block()
            .inline_asm(node.is_memory_safe(), self)
            .into();
        let body = node.body();
        let mut scope = AssemblyScope::new(self);
        scope.body(block, |scope| scope.assembly_body(&body));
    }
}

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// The assembly block's body. Every Yul function the block declares, at any nesting depth, is
    /// emitted first: Yul calls resolve regardless of definition order, and `sol.inline_asm` is one
    /// flat symbol table.
    fn assembly_body(&mut self, node: &YulBlock) {
        let definitions = Self::function_definitions(node);
        for definition in &definitions {
            self.declare_function(definition);
        }
        for definition in &definitions {
            self.define_function(definition);
        }
        self.yul_statements(node);
    }

    /// Records `definition`'s signature, so a call site reaching it resolves whether or not the
    /// definition has been emitted yet.
    fn declare_function(&mut self, definition: &YulFunctionDefinition) {
        let signature = YulFunction::new(
            definition.name().name().to_owned(),
            definition.parameters().len(),
            definition.returns().map_or(0, |returns| returns.len()),
        );
        self.functions.insert(definition.node_id(), signature);
    }

    /// Emits `definition`'s body into its `yul.func`. Parameters and return variables are slots like
    /// any Yul variable: the parameter slot takes the incoming block argument, the return slot the
    /// zero a Yul return variable defaults to.
    fn define_function(&mut self, definition: &YulFunctionDefinition) {
        let entry = self
            .signature(definition.node_id())
            .define(self, self.current_block().into());

        let parameters: Vec<_> = definition.parameters().iter().collect();
        let return_names: Vec<_> = definition
            .returns()
            .map(|returns| returns.iter().collect())
            .unwrap_or_default();
        let body = definition.body();

        self.body(entry, |scope| {
            for (index, parameter) in parameters.iter().enumerate() {
                let argument = entry.argument(index);
                scope.bind(parameter.node_id(), argument);
            }
            let returns: Vec<Slot> = return_names
                .iter()
                .map(|name| scope.bind_zero(name.node_id()))
                .collect();

            scope.in_function(returns, |scope| {
                scope.yul_statements(&body);
                if !scope.current_block().is_terminated() {
                    scope.func_return();
                }
            });
        });
    }

    /// Emits `yul.func_return` carrying the current words of the enclosing Yul function's return
    /// variables. Both `leave` and the fall-through end of a body reach it.
    pub fn func_return(&mut self) {
        let returns = self.returns.clone();
        let operands: Vec<_> = returns.iter().map(|slot| slot.load(self)).collect();
        self.current_block().func_return(&operands, self);
    }

    /// Every Yul function definition in the block tree, outermost first. Nested definitions are
    /// collected too: Yul allows a function inside a function, and both are siblings in the symbol
    /// table.
    fn function_definitions(node: &YulBlock) -> Vec<YulFunctionDefinition> {
        let mut definitions = Vec::new();
        Self::collect_function_definitions(node, &mut definitions);
        definitions
    }

    /// Appends the function definitions `node` and its nested blocks declare to `definitions`.
    fn collect_function_definitions(node: &YulBlock, definitions: &mut Vec<YulFunctionDefinition>) {
        for statement in node.statements().iter() {
            match statement {
                YulStatement::YulFunctionDefinition(definition) => {
                    let body = definition.body();
                    definitions.push(definition);
                    Self::collect_function_definitions(&body, definitions);
                }
                YulStatement::YulBlock(block) => {
                    Self::collect_function_definitions(&block, definitions);
                }
                YulStatement::YulIfStatement(statement) => {
                    Self::collect_function_definitions(&statement.body(), definitions);
                }
                YulStatement::YulForStatement(statement) => {
                    Self::collect_function_definitions(&statement.initialization(), definitions);
                    Self::collect_function_definitions(&statement.body(), definitions);
                    Self::collect_function_definitions(&statement.iterator(), definitions);
                }
                YulStatement::YulSwitchStatement(statement) => {
                    for case in statement.value_cases().iter() {
                        Self::collect_function_definitions(&case.body(), definitions);
                    }
                    if let Some(case) = statement.default_case() {
                        Self::collect_function_definitions(&case.body(), definitions);
                    }
                }
                _ => {}
            }
        }
    }
}
