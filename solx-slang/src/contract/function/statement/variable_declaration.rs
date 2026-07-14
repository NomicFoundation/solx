//!
//! Variable declaration statements, single and tuple-deconstructing.
//!

use solx_mlir::Value;

use crate::contract::function::expression::Expression;

dispatch!(
    /// A variable declaration, single or tuple-deconstructing.
    VariableDeclarationTarget(VariableDeclarationTarget) -> Effect |node, scope| {
        SingleTypedDeclaration,
        MultiTypedDeclaration,
    }
);

codegen!(
    VariableDeclarationStatement -> Effect |node, scope| {
        VariableDeclarationTarget::emit(&node.target(), scope);
    }

    /// A single-typed variable declaration. An explicit initializer is evaluated before the slot
    /// is allocated, matching solc's order; an absent one default-initializes the freshly
    /// allocated slot, which integers zero-fill.
    SingleTypedDeclaration -> Effect |node, scope| {
        let name = node.declaration().name().name();
        let declared_type = codegen!(@result_type VariableDeclaration, node.declaration(), scope);
        match node.value() {
            Some(initializer) => {
                let value = Expression::emit(&initializer, scope).coerce(declared_type, scope);
                scope.define_local(name, declared_type, |_context| value);
            }
            None if declared_type.is_integer() => {
                scope.define_local(name, declared_type, |scope| {
                    Value::zero(declared_type, scope)
                });
            }
            None => unimplemented!("zero-initialization for non-integer type {declared_type}"),
        }
    }

    /// A multi-typed variable declaration, deconstructing a tuple or call.
    MultiTypedDeclaration -> Effect |node, scope| {
        let values = Expression::emit_values(&node.value(), scope);
        for (member, value) in node.elements().iter().zip(values) {
            let Some(declaration) = member.member() else {
                continue;
            };
            let name = declaration.name().name();
            let declared_type = codegen!(@result_type VariableDeclaration, declaration, scope);
            scope.define_local(name, declared_type, |scope| {
                value.coerce(declared_type, scope)
            });
        }
    }
);
