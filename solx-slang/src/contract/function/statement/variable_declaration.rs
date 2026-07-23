//!
//! Variable declaration statements, single and tuple-destructuring.
//!

use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// A variable declaration statement, delegating to its target.
    pub fn variable_declaration_statement(&mut self, node: &VariableDeclarationStatement) {
        self.variable_declaration_target(&node.target());
    }

    /// A variable declaration target, single or tuple-destructuring.
    pub fn variable_declaration_target(&mut self, node: &VariableDeclarationTarget) {
        match node {
            VariableDeclarationTarget::SingleTypedDeclaration(inner) => {
                self.single_typed_declaration(inner)
            }
            VariableDeclarationTarget::MultiTypedDeclaration(inner) => {
                self.multi_typed_declaration(inner)
            }
        }
    }

    /// A single-typed variable declaration. An explicit initializer is evaluated before the slot is
    /// allocated, matching solc's order; an absent one default-initializes the slot to the type's
    /// default value.
    pub fn single_typed_declaration(&mut self, node: &SingleTypedDeclaration) {
        let declared_type = self.typing(node.declaration().get_type());
        match node.value() {
            Some(initializer) => {
                let value = self.converted(&initializer, declared_type);
                self.define_local(node.declaration().name().name(), declared_type, |_scope| {
                    value
                });
            }
            None => {
                self.define_local(node.declaration().name().name(), declared_type, |scope| {
                    Value::default_initialized(declared_type, scope)
                });
            }
        }
    }

    /// A multi-typed variable declaration, destructuring a tuple or a call through `converted_values`,
    /// so a string-literal tuple element folds to its bytes-like constant. A blank slot evaluates its
    /// operand but binds nothing.
    pub fn multi_typed_declaration(&mut self, node: &MultiTypedDeclaration) {
        let elements = node.elements();
        let targets: Vec<_> = elements
            .iter()
            .map(|member| {
                member
                    .member()
                    .map(|declaration| self.typing(declaration.get_type()))
            })
            .collect();
        let values = self.converted_values(&node.value(), &targets);
        for ((member, target), value) in elements.iter().zip(targets).zip(values) {
            if let Some(declaration) = member.member() {
                let target = target.expect("a bound slot has a declared type");
                self.define_local(declaration.name().name(), target, |_scope| value);
            }
        }
    }
}
