//!
//! The contextual keyword expressions: `this` as a value, and the lookup a `super` member resolves
//! by.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type;

use solx_mlir::Value;

use crate::scope::contract::Lookup;
use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `this` keyword: the enclosing contract as a value.
    pub fn this_value(&mut self) -> Value<'context> {
        Value::this(
            self.current_contract_type
                .expect("sol.this emitted outside a contract"),
            self,
        )
    }

    /// The lookup resolving a `super` member access, when `node`'s operand is `super`: after the
    /// contract the keyword is written in, which Slang anchors it at, since the object being
    /// compiled decides the linearisation.
    pub fn super_lookup(node: &MemberAccessExpression) -> Option<Lookup> {
        let Expression::SuperKeyword(keyword) = Self::unparenthesized(&node.operand()) else {
            return None;
        };
        let Some(Type::Contract(contract_type)) = keyword.anchor() else {
            unreachable!(
                "slang anchors `super` at the contract or interface it is written in, and no interface body is lowered"
            );
        };
        let Definition::Contract(anchor) = contract_type.definition() else {
            unreachable!("slang ContractType always references a Contract definition");
        };
        Some(Lookup::Super(anchor))
    }
}
