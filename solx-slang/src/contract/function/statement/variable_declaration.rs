//!
//! Variable declaration statements, single and tuple-deconstructing.
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

    /// A variable declaration target, single or tuple-deconstructing.
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
    /// allocated, matching solc's order; an absent one default-initializes the freshly allocated
    /// slot, which integers zero-fill.
    pub fn single_typed_declaration(&mut self, node: &SingleTypedDeclaration) {
        let declared_type = self.typing(node.declaration().get_type());
        match node.value() {
            Some(initializer) => {
                let value = self.coerced(&initializer, declared_type);
                self.define_local(node.declaration().name().name(), declared_type, |_scope| {
                    value
                });
            }
            None if declared_type.is_integer() => {
                self.define_local(node.declaration().name().name(), declared_type, |scope| {
                    Value::zero(declared_type, scope)
                });
            }
            None => unimplemented!("zero-initialization for non-integer type {declared_type}"),
        }
    }

    /// A multi-typed variable declaration, deconstructing a tuple or call.
    pub fn multi_typed_declaration(&mut self, node: &MultiTypedDeclaration) {
        let values = self.expression_values(&node.value());
        for (member, value) in node.elements().iter().zip(values) {
            let Some(declaration) = member.member() else {
                continue;
            };
            let declared_type = self.typing(declaration.get_type());
            self.define_local(declaration.name().name(), declared_type, |scope| {
                value.coerce(declared_type, scope)
            });
        }
    }
}
