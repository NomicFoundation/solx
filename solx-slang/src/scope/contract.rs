//!
//! The contract scope: the enclosing source unit scope, the `sol.contract` the members are
//! defined into, and the object whose hierarchy a member resolves against.
//!

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocation;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::VirtualTarget;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Contract;
use solx_mlir::Function;

use crate::contract::constructor::ConstructorBuilder;
use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

/// The contract scope: the enclosing source unit scope, the `sol.contract` the members are
/// defined into, and the object whose hierarchy a member resolves against.
pub struct ContractScope<'source_unit, 'context> {
    /// The source unit scope this contract is lowered within.
    pub source_unit: &'source_unit mut SourceUnitScope<'context>,
    /// The `sol.contract` the members are declared and defined into.
    pub contract: Contract<'context>,
    /// The object being emitted, whose linearisation resolves the references its bodies make.
    pub object: &'source_unit Object,
    /// The definition ids of the functions defined so far.
    pub defined_functions: HashSet<NodeId>,
    /// The state-variable slots keyed by definition id.
    pub storage_layout: HashMap<NodeId, StorageSlot>,
    /// Mutable state for emitting the object's constructor chain.
    pub constructor: ConstructorBuilder<'context>,
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Opens a contract scope within `source_unit`.
    pub fn new(
        source_unit: &'source_unit mut SourceUnitScope<'context>,
        contract: Contract<'context>,
        object: &'source_unit Object,
    ) -> Self {
        Self {
            source_unit,
            contract,
            object,
            defined_functions: HashSet::new(),
            storage_layout: object.storage_layout(),
            constructor: ConstructorBuilder::new(object.contracts()),
        }
    }

    /// Opens the function scope around `emit`: whether the frame is a constructor, a fresh
    /// variable environment, the declared return types a `return` converts to, and checked
    /// arithmetic, with the MLIR cursor on `entry` for the body's duration.
    pub fn function(
        &mut self,
        entry: Block<'context>,
        is_constructor: bool,
        signature: &Function<'context>,
        emit: impl FnOnce(&mut FunctionScope<'_, '_, 'context>),
    ) {
        let enclosing = self.source_unit.mlir.current_block.replace(entry);
        emit(&mut FunctionScope::new(
            self,
            is_constructor,
            &signature.function_type.results,
        ));
        self.source_unit.mlir.current_block = enclosing;
    }

    /// The function a bare name runs in this object: the most-derived override of its hierarchy,
    /// or the function itself when it is free or a library's, which nothing overrides.
    pub fn virtual_function(&self, function: &FunctionDefinition) -> FunctionDefinition {
        let Object::Contract(node) = &self.object else {
            return function.clone();
        };
        match node.resolve_virtual(function) {
            Some(VirtualTarget::Function(function)) => function,
            Some(VirtualTarget::Getter(_)) => {
                unreachable!("a getter overrides an external function, which no bare name calls")
            }
            None => function.clone(),
        }
    }

    /// The function a `super` member runs in this object: the nearest implemented override after
    /// `enclosing_contract` in its linearisation, which only a contract has.
    pub fn super_function(
        &self,
        function: &FunctionDefinition,
        enclosing_contract: &ContractDefinition,
    ) -> FunctionDefinition {
        let Object::Contract(node) = &self.object else {
            unreachable!("`super` is written in a contract alone");
        };
        node.resolve_super(function, enclosing_contract)
            .expect("a `super` call resolves to an implemented base function")
    }

    /// The modifier a modifier-list entry runs in this object: the most-derived override of its
    /// hierarchy for a bare name, or the declaration itself when the name is qualified or the
    /// modifier is a library's, which nothing overrides.
    pub fn invoked_modifier(
        &self,
        invocation: &ModifierInvocation,
        declaration: FunctionDefinition,
    ) -> FunctionDefinition {
        match (&self.object, declaration.enclosing_definition()) {
            (Object::Contract(node), Some(Definition::Contract(_))) => {
                node.resolve_modifier(invocation).expect(
                    "a contract's modifier resolves in the hierarchy of a contract invoking it",
                )
            }
            _ => declaration,
        }
    }
}

impl<'source_unit, 'context> Deref for ContractScope<'source_unit, 'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.source_unit.mlir
    }
}
