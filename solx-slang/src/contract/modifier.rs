//!
//! Modifier emission: the `sol.modifier` an invocation names and the invocations a function
//! carries.
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;

use crate::scope::contract::ContractScope;
use crate::scope::function::FunctionScope;

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Defines `definition`'s `sol.modifier` in the contract body at its first naming, binding its
    /// parameters into a fresh frame with nothing to return, and hands back the symbol an
    /// invocation names; a later naming hands back the same.
    fn modifier_definition(&mut self, definition: &FunctionDefinition) -> String {
        let signature = self.source_unit.function_signature(definition);
        if !self.defined_members.insert(definition.node_id()) {
            return signature.mlir_name;
        }
        let body = definition
            .body()
            .expect("slang admits a qualified invocation of a modifier without a body");
        let entry = self.contract.define_modifier(
            &signature.mlir_name,
            signature.function_type.parameters.clone(),
            self,
        );
        self.function(entry, false, &signature, |scope| {
            scope.bind_parameters(&definition.parameters(), &entry.arguments());
            scope.statements(&body.statements());
            if !scope.current_block().is_terminated() {
                scope.current_block().r#return(&[], scope);
            }
        });
        signature.mlir_name
    }
}

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Emits `function`'s modifier invocations in source order, each a `sol.modifier_invocation`
    /// whose region evaluates the arguments and yields them: a bare name runs the modifier the
    /// object dispatches for the declaration, a qualified name the declaration itself. An entry
    /// naming a base is a base-constructor call, which the constructor chain consumes.
    pub fn modifier_invocations(&mut self, function: &FunctionDefinition) {
        for invocation in function.attributes().modifier_invocations().iter() {
            let declaration = match invocation.name().resolve_to_definition() {
                Some(Definition::Modifier(declaration)) => declaration,
                Some(Definition::Contract(_) | Definition::Interface(_)) => continue,
                _ => unreachable!("a modifier-list entry names a modifier or a base"),
            };
            let definition = self.contract.resolve_modifier(&invocation, &declaration);
            let symbol = self.contract.modifier_definition(&definition);
            let arguments_block = self.current_block().modifier_invocation(&symbol, self);
            self.region(arguments_block, |scope| {
                let values: Vec<_> = invocation
                    .arguments()
                    .map(|arguments| {
                        scope.arguments_declaration(
                            &ArgumentsDeclaration::PositionalArguments(arguments),
                            &definition.parameters(),
                        )
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect();
                scope.current_block().r#yield(&values, scope);
            });
        }
    }
}
