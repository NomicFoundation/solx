//!
//! The contract scope: the enclosing source unit scope, the `sol.contract` the members are
//! defined into, and the object whose hierarchy a member resolves against.
//!

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocation;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::VirtualTarget;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Contract;
use solx_mlir::Function;

use crate::contract::constructor_chain::ConstructorChain;
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
    /// The definition ids of the functions and modifiers defined so far, each inserted before its
    /// body is emitted, since a body may reach a function the modifier decorates.
    pub defined_members: HashSet<NodeId>,
    /// The state-variable slots keyed by definition id.
    pub storage_layout: HashMap<NodeId, StorageSlot>,
    /// The constructors the object's creation runs and the argument lists they pass along.
    pub chain: &'source_unit ConstructorChain,
    /// The definition ids a selector is emitted for.
    pub dispatched_functions: HashSet<NodeId>,
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Opens a contract scope within `source_unit`.
    pub fn new(
        source_unit: &'source_unit mut SourceUnitScope<'context>,
        contract: Contract<'context>,
        object: &'source_unit Object,
        chain: &'source_unit ConstructorChain,
    ) -> Self {
        Self {
            source_unit,
            contract,
            object,
            defined_members: HashSet::new(),
            storage_layout: object.storage_layout(),
            chain,
            dispatched_functions: object
                .functions()
                .iter()
                .map(|function| function.node_id())
                .collect(),
        }
    }

    /// Opens the function scope around `emit`: where the frame sits in the constructor chain, a
    /// fresh variable environment, the declared return types a `return` converts to, and checked
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

    /// The function a bare name runs in this object: the most-derived override of its hierarchy.
    /// A library's functions are never overridden.
    pub fn virtual_function(&self, function: &FunctionDefinition) -> FunctionDefinition {
        let Object::Contract(node) = &self.object else {
            return function.clone();
        };
        match node.resolve_virtual(function) {
            VirtualTarget::Function(function) => function,
            VirtualTarget::Getter(_) => {
                unreachable!("a getter overrides an external function, which no bare name calls")
            }
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
    }

    /// The modifier the modifier-list entry `invocation`, naming `declaration`, runs in this
    /// object: in a contract, Slang's answer, the most-derived override of a virtual declaration
    /// for a bare name and the declaration for a qualified one; in a library, the declaration,
    /// which nothing overrides.
    pub fn resolve_modifier(
        &self,
        invocation: &ModifierInvocation,
        declaration: &FunctionDefinition,
    ) -> FunctionDefinition {
        match &self.object {
            Object::Contract(node) => node.resolve_modifier(invocation, declaration),
            Object::Library(_) => declaration.clone(),
        }
    }
}

impl<'source_unit, 'context> Deref for ContractScope<'source_unit, 'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.source_unit.mlir
    }
}
