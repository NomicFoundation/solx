//!
//! The function scope: the enclosing contract scope, the lexical variable environment, the declared
//! return types, and the checked-arithmetic flag, together with the frame combinators every
//! lowering threads through.
//!

use std::ops::Deref;

use slang_solidity_v2::ast::Type;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::contract::ContractScope;

/// The function scope: the enclosing contract scope, the lexical variable environment, the declared
/// return types a `return` converts to, and whether arithmetic is checked at the current position.
pub struct FunctionScope<'contract, 'source_unit, 'context> {
    /// The contract scope this function body is lowered within.
    pub contract: &'contract mut ContractScope<'source_unit, 'context>,
    /// The lexically scoped variable bindings.
    pub environment: Environment<'context>,
    /// The declared return types a `return` converts to.
    pub return_types: Vec<MlirType<'context>>,
    /// Whether arithmetic reverts on overflow at the current position.
    pub checked: bool,
}

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Opens a function scope within `contract` with the given declared return types.
    pub fn new(
        contract: &'contract mut ContractScope<'source_unit, 'context>,
        return_types: Vec<MlirType<'context>>,
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
        element_type: MlirType<'context>,
        initializer: impl FnOnce(&mut Self) -> Value<'context>,
    ) -> Place<'context> {
        let pointer = Place::stack(element_type, self);
        pointer.store(initializer(self), self);
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

    /// Branches `condition` into one `result_type` pointer and loads the merge. The pointer is first
    /// stored with whatever `initializer` yields, then each arm that yields a value stores it,
    /// converted to `result_type`, while an arm that yields none leaves the initializing value in
    /// place. The shared lowering of `?:` (no initializer, both arms store) and the short-circuit
    /// `&&` / `||` (initialized, the short-circuiting arm empty).
    pub fn branch_value(
        &mut self,
        condition: Value<'context>,
        result_type: MlirType<'context>,
        initializer: impl FnOnce(&mut Self) -> Option<Value<'context>>,
        then: impl FnOnce(&mut Self) -> Option<Value<'context>>,
        r#else: impl FnOnce(&mut Self) -> Option<Value<'context>>,
    ) -> Value<'context> {
        let pointer = Place::stack(result_type, self);
        if let Some(value) = initializer(self) {
            pointer.store(value, self);
        }
        let (then_block, else_block) = self.current_block().branch_with_else(condition, self);
        self.region(then_block, |scope| {
            if let Some(value) = then(scope) {
                pointer.store(value.convert(result_type, scope), scope);
            }
        });
        self.region(else_block, |scope| {
            if let Some(value) = r#else(scope) {
                pointer.store(value.convert(result_type, scope), scope);
            }
        });
        pointer.load(result_type, self)
    }

    /// Resolves a Slang semantic type through the source unit scope.
    pub fn resolve_type(
        &self,
        node: &Type,
        inherited_location: Option<solx_utils::DataLocation>,
    ) -> MlirType<'context> {
        self.contract.source_unit.resolve(node, inherited_location)
    }

    /// The binder's typing of a node, resolved through the source unit scope.
    pub fn typing(&self, slang_type: Option<Type>) -> MlirType<'context> {
        self.contract.source_unit.typing(slang_type)
    }

    /// The MLIR pointer type for a value of this Slang type through the source unit scope.
    pub fn pointer_type(
        &self,
        node: &Type,
        element_type: MlirType<'context>,
        base_location: solx_utils::DataLocation,
    ) -> MlirType<'context> {
        self.contract
            .source_unit
            .pointer(node, element_type, base_location)
    }
}

impl<'contract, 'source_unit, 'context> Deref for FunctionScope<'contract, 'source_unit, 'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.contract.source_unit.mlir
    }
}
