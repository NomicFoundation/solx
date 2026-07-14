//!
//! The emission scope strata. A source unit scope owns the MLIR context, a contract scope borrows
//! it to hold the enclosing contract's storage layout and body, and a function scope borrows that
//! to hold a body's variable environment. Each scope dereferences to the MLIR context it emits ops
//! into; its enclosing scope is reached explicitly through the borrow it owns.
//!

use std::collections::HashMap;
use std::ops::Deref;

use slang_solidity_v2::ast::NodeId;

use solx_mlir::Block;
use solx_mlir::Context as MlirContext;
use solx_mlir::Environment;
use solx_mlir::Function as MlirFunction;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::contract::storage_slot::StorageSlot;

/// The source unit scope: the owned MLIR context that every nested scope emits into.
pub struct SourceUnitScope<'context> {
    /// The owned MLIR context, surrendered by the conversion into it.
    mlir: MlirContext<'context>,
}

impl<'context> SourceUnitScope<'context> {
    /// Wraps the MLIR context for one source unit's emission.
    pub fn new(mlir: MlirContext<'context>) -> Self {
        Self { mlir }
    }

    /// Opens the contract scope around `emit`: the storage layout and body an enclosed function
    /// resolves against, with the `this` type installed on the MLIR context for its duration.
    pub fn contract(
        &mut self,
        storage_layout: HashMap<NodeId, StorageSlot>,
        contract_type: Type<'context>,
        body: Block<'context>,
        emit: impl FnOnce(&mut ContractScope<'_, 'context>),
    ) {
        self.mlir.current_contract_type = Some(contract_type);
        {
            let mut contract = ContractScope::new(self, storage_layout, body);
            emit(&mut contract);
        }
        self.mlir.current_contract_type = None;
    }

    /// Registers a function signature keyed by its AST definition id.
    pub fn register_function_signature(
        &mut self,
        definition_node_id: NodeId,
        mlir_name: String,
        parameter_types: Vec<Type<'context>>,
        return_types: Vec<Type<'context>>,
    ) {
        self.mlir.register_function_signature(
            definition_node_id,
            mlir_name,
            parameter_types,
            return_types,
        );
    }

    /// The pre-registered signature of `definition_node_id`'s function.
    pub fn function_signature(&self, definition_node_id: NodeId) -> MlirFunction<'context> {
        self.mlir
            .function_signatures
            .get(&definition_node_id)
            .cloned()
            .expect("the contract lowering pre-registers every function")
    }
}

impl<'context> Deref for SourceUnitScope<'context> {
    type Target = MlirContext<'context>;

    fn deref(&self) -> &Self::Target {
        &self.mlir
    }
}

impl<'context> From<SourceUnitScope<'context>> for MlirContext<'context> {
    fn from(scope: SourceUnitScope<'context>) -> Self {
        scope.mlir
    }
}

/// The contract scope: the enclosing source unit scope, the state-variable storage layout a member
/// resolves against, and the block the contract's functions are defined into.
pub struct ContractScope<'source_unit, 'context> {
    /// The source unit scope this contract is lowered within.
    source_unit: &'source_unit mut SourceUnitScope<'context>,
    /// The state-variable slots keyed by definition id.
    storage_layout: HashMap<NodeId, StorageSlot>,
    /// The block the contract's `sol.func`s are defined into.
    contract_body: Block<'context>,
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Opens a contract scope within `source_unit`.
    pub fn new(
        source_unit: &'source_unit mut SourceUnitScope<'context>,
        storage_layout: HashMap<NodeId, StorageSlot>,
        contract_body: Block<'context>,
    ) -> Self {
        Self {
            source_unit,
            storage_layout,
            contract_body,
        }
    }

    /// Opens the function scope around `emit`: a fresh variable environment, the declared return
    /// types a `return` coerces to, and checked arithmetic, with the MLIR cursor on `entry` for the
    /// body's duration.
    pub fn function(
        &mut self,
        entry: Block<'context>,
        return_types: Vec<Type<'context>>,
        emit: impl FnOnce(&mut FunctionScope<'_, '_, 'context>),
    ) {
        let enclosing = self.source_unit.mlir.current_block.replace(entry);
        {
            let mut function = FunctionScope::new(self, return_types);
            emit(&mut function);
        }
        self.source_unit.mlir.current_block = enclosing;
    }

    /// The enclosing source unit scope.
    pub fn source_unit(&self) -> &SourceUnitScope<'context> {
        self.source_unit
    }

    /// The state-variable slots keyed by definition id.
    pub fn storage_layout(&self) -> &HashMap<NodeId, StorageSlot> {
        &self.storage_layout
    }

    /// The block the contract's functions are defined into.
    pub fn contract_body(&self) -> Block<'context> {
        self.contract_body
    }
}

impl<'source_unit, 'context> Deref for ContractScope<'source_unit, 'context> {
    type Target = MlirContext<'context>;

    fn deref(&self) -> &Self::Target {
        &self.source_unit.mlir
    }
}

/// The function scope: the enclosing contract scope, the lexical variable environment, the declared
/// return types a `return` coerces to, and whether arithmetic is checked at the current position.
pub struct FunctionScope<'contract, 'source_unit, 'context> {
    /// The contract scope this function body is lowered within.
    contract: &'contract mut ContractScope<'source_unit, 'context>,
    /// The lexically scoped variable bindings.
    environment: Environment<'context>,
    /// The declared return types a `return` coerces to.
    return_types: Vec<Type<'context>>,
    /// Whether arithmetic reverts on overflow at the current position.
    checked: bool,
}

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Opens a function scope within `contract` with the given declared return types.
    pub fn new(
        contract: &'contract mut ContractScope<'source_unit, 'context>,
        return_types: Vec<Type<'context>>,
    ) -> Self {
        Self {
            contract,
            environment: Environment::new(),
            return_types,
            checked: true,
        }
    }

    /// Binds a named local: allocates its stack pointer, stores the value its initializer yields,
    /// and defines the binding in the current scope. The initializer runs after the allocation so
    /// the slot precedes the value that initializes it, matching solc's emission order.
    pub fn define_local(
        &mut self,
        name: String,
        element_type: Type<'context>,
        initializer: impl FnOnce(&mut Self) -> Value<'context>,
    ) -> Place<'context> {
        let pointer = Place::stack(element_type, self);
        let value = initializer(self);
        pointer.store(value, self);
        self.environment
            .define_variable(name, pointer, element_type);
        pointer
    }

    /// Emits with unchecked arithmetic, restoring the enclosing flag afterwards.
    pub fn unchecked(&mut self, emit: impl FnOnce(&mut Self)) {
        let enclosing = std::mem::replace(&mut self.checked, false);
        emit(self);
        self.checked = enclosing;
    }

    /// Runs `emit` in a nested lexical scope, discarding the bindings it introduces.
    pub fn nested(&mut self, emit: impl FnOnce(&mut Self)) {
        self.environment.enter_scope();
        emit(self);
        self.environment.exit_scope();
    }

    /// Emits into `block`, appends the implicit `sol.yield` if the emitted code did not terminate
    /// it, and restores the cursor to the enclosing block.
    pub fn region(&mut self, block: Block<'context>, emit: impl FnOnce(&mut Self)) {
        let enclosing = self.contract.source_unit.mlir.current_block.replace(block);
        emit(self);
        let end = self.current_block();
        if !end.is_terminated() {
            end.r#yield(&[], self);
        }
        self.contract.source_unit.mlir.current_block = enclosing;
    }

    /// Like [`Self::region`], terminated by `sol.condition` on the closure's value's truthiness.
    pub fn condition_region(
        &mut self,
        block: Block<'context>,
        emit: impl FnOnce(&mut Self) -> Value<'context>,
    ) {
        self.region(block, |function| {
            let condition = emit(function).is_nonzero(function);
            function.current_block().condition(condition, function);
        });
    }

    /// The enclosing contract scope.
    pub fn contract(&self) -> &ContractScope<'source_unit, 'context> {
        self.contract
    }

    /// The lexically scoped variable bindings.
    pub fn environment(&self) -> &Environment<'context> {
        &self.environment
    }

    /// The declared return types a `return` coerces to.
    pub fn return_types(&self) -> &[Type<'context>] {
        &self.return_types
    }

    /// Whether arithmetic reverts on overflow at the current position.
    pub fn checked(&self) -> bool {
        self.checked
    }
}

impl<'contract, 'source_unit, 'context> Deref for FunctionScope<'contract, 'source_unit, 'context> {
    type Target = MlirContext<'context>;

    fn deref(&self) -> &Self::Target {
        &self.contract.source_unit.mlir
    }
}
