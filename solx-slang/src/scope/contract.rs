//!
//! The contract scope: the enclosing source unit scope, the `sol.contract` the members are
//! defined into, the object whose hierarchy a member resolves against, and the code segment it
//! lowers.
//!

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocation;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::ast::VirtualTarget;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Contract;
use solx_mlir::Function;
use solx_utils::CodeSegment;

use crate::contract::constructor::ConstructorBuilder;
use crate::contract::indirect_callees::IndirectCallees;
use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

/// The contract scope: the enclosing source unit scope, the `sol.contract` the members are
/// defined into, the object whose hierarchy a member resolves against, and the code segment it
/// lowers.
pub struct ContractScope<'source_unit, 'context> {
    /// The source unit scope this contract is lowered within.
    pub source_unit: &'source_unit mut SourceUnitScope<'context>,
    /// The `sol.contract` the members are declared and defined into.
    pub contract: Contract<'context>,
    /// The object being emitted, whose linearisation resolves the references its bodies make.
    pub object: &'source_unit Object,
    /// The code segment being emitted: the deploy code, which runs the constructor, or the
    /// runtime code, which dispatches calls.
    pub segment: CodeSegment,
    /// The definition ids of the functions defined so far.
    pub defined_functions: HashSet<NodeId>,
    /// The deploy code's indirect callees, taken by this segment or reachable from it.
    pub indirect_callees: IndirectCallees<'source_unit>,
    /// The state-variable slots keyed by definition id, computed once for both segments.
    pub storage_layout: &'source_unit HashMap<NodeId, StorageSlot>,
    /// Mutable state for emitting the object's constructor chain.
    pub constructor: ConstructorBuilder<'context>,
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Opens a contract scope within `source_unit`, with the object's `storage_layout` and the
    /// deploy code's indirect callees, which fix the code segment it lowers.
    pub fn new(
        source_unit: &'source_unit mut SourceUnitScope<'context>,
        contract: Contract<'context>,
        object: &'source_unit Object,
        storage_layout: &'source_unit HashMap<NodeId, StorageSlot>,
        indirect_callees: IndirectCallees<'source_unit>,
    ) -> Self {
        Self {
            source_unit,
            contract,
            object,
            segment: indirect_callees.segment(),
            defined_functions: HashSet::new(),
            indirect_callees,
            storage_layout,
            constructor: ConstructorBuilder::new(object.contracts()),
        }
    }

    /// Opens the function scope around `emit`: a fresh
    /// variable environment, the declared return types a `return` converts to, and checked
    /// arithmetic, with the MLIR cursor on `entry` for the body's duration. The location cursor is
    /// the caller's, the function's own node; each part of the body with a node of its own narrows
    /// it.
    pub fn function(
        &mut self,
        entry: Block<'context>,
        signature: &Function<'context>,
        emit: impl FnOnce(&mut FunctionScope<'_, '_, 'context>),
    ) {
        let enclosing = self.source_unit.mlir.current_block.replace(entry);
        emit(&mut FunctionScope::new(
            self,
            &signature.function_type.results,
        ));
        self.source_unit.mlir.current_block = enclosing;
    }

    /// Runs `emit` with the location cursor on `node`'s first byte, so the ops it emits carry it,
    /// and restores the enclosing cursor afterwards. The node's source range is read only when the
    /// segment requested debug info.
    pub fn at_node<R>(&mut self, node: &impl NodeLocation, emit: impl FnOnce(&mut Self) -> R) -> R {
        let location = self.source_unit.debug_locations.location(node);
        let enclosing = std::mem::replace(&mut self.source_unit.mlir.current_location, location);
        let result = emit(self);
        self.source_unit.mlir.current_location = enclosing;
        result
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

    /// The modifier a modifier-list entry runs in this object, or `None` for an entry naming a
    /// base, which is a base-constructor call. A contract's modifier is Slang's dispatch answer:
    /// the most-derived override for a bare name, the declaration for a qualified one. A
    /// library's modifier, which that dispatch does not cover and nothing overrides, is the
    /// declaration.
    pub fn invoked_modifier(&self, invocation: &ModifierInvocation) -> Option<FunctionDefinition> {
        let declaration = match invocation.name().resolve_to_definition() {
            Some(Definition::Modifier(declaration)) => declaration,
            Some(Definition::Contract(_) | Definition::Interface(_)) => return None,
            _ => unreachable!("a modifier-list entry names a modifier or a base"),
        };
        Some(match (&self.object, declaration.enclosing_definition()) {
            (Object::Contract(node), Some(Definition::Contract(_))) => {
                node.resolve_modifier(invocation).expect(
                    "a contract's modifier resolves in the hierarchy of a contract invoking it",
                )
            }
            _ => declaration,
        })
    }
}

impl<'source_unit, 'context> Deref for ContractScope<'source_unit, 'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.source_unit.mlir
    }
}
