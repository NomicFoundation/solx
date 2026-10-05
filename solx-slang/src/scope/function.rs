//!
//! The function scope: the enclosing contract scope, the
//! lexical variable environment, the declared return types, and the checked-arithmetic flag,
//! together with the frame combinators every lowering threads through.
//!

use std::ops::Deref;

use melior::ir::Location;
use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::ast::Type;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::assembly::AssemblyScope;
use crate::scope::contract::ContractScope;

/// The function scope: the enclosing contract scope, the
/// lexical variable environment, the declared return types a `return` converts to, and whether
/// arithmetic is checked at the current position.
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
        return_types: &[MlirType<'context>],
    ) -> Self {
        Self {
            contract,
            environment: Environment::new(),
            return_types: return_types.to_vec(),
            checked: true,
        }
    }

    /// Binds a named local: allocates its stack pointer, stores the value its initializer yields,
    /// and defines the binding in the current scope. The initializer runs after the allocation so
    /// the slot precedes the value that initializes it.
    pub fn define_local(
        &mut self,
        name: &str,
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
    pub fn nested<R>(&mut self, emit: impl FnOnce(&mut Self) -> R) -> R {
        self.environment.enter_scope();
        let result = emit(self);
        self.environment.exit_scope();
        result
    }

    /// Runs `emit` with the location cursor on `node`'s first byte, so the ops it emits carry it,
    /// and restores the enclosing cursor afterwards. The node's source range is read only when the
    /// segment requested debug info.
    pub fn at_node<R>(&mut self, node: &impl NodeLocation, emit: impl FnOnce(&mut Self) -> R) -> R {
        let location = self.contract.source_unit.debug_locations.location(node);
        self.at(location, emit)
    }

    /// Like [`Self::at_node`], with the location cursor on `node`'s last byte: the closing brace of
    /// a body, where the implicit return of one that falls through belongs.
    pub fn at_node_end<R>(
        &mut self,
        node: &impl NodeLocation,
        emit: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let location = self.contract.source_unit.debug_locations.location_end(node);
        self.at(location, emit)
    }

    /// Opens the assembly scope around `emit`: the Yul bindings an inline-assembly block
    /// introduces, with the MLIR cursor on `body` - the `sol.inline_asm` region, which is also the
    /// symbol table its Yul functions are emitted into - for the block's duration.
    pub fn assembly(
        &mut self,
        body: Block<'context>,
        emit: impl FnOnce(&mut AssemblyScope<'_, '_, '_, 'context>),
    ) {
        let enclosing = self.contract.source_unit.mlir.current_block.replace(body);
        emit(&mut AssemblyScope::new(self, body));
        self.contract.source_unit.mlir.current_block = enclosing;
    }

    /// Emits into `block`, appends the implicit `sol.yield` if the emitted code did not terminate
    /// it, and restores the cursor to the enclosing block. The yield carries the location of the
    /// construct opening the region, since every node lowered inside has restored the location
    /// cursor by then.
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
        &mut self,
        node: &Type,
        inherited_location: Option<solx_utils::DataLocation>,
    ) -> MlirType<'context> {
        self.contract.source_unit.resolve(node, inherited_location)
    }

    /// The binder's typing of a node, resolved through the source unit scope.
    pub fn typing(&mut self, slang_type: Option<Type>) -> MlirType<'context> {
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

    /// Runs `emit` with the location cursor on `location`, restoring the enclosing cursor
    /// afterwards.
    fn at<R>(&mut self, location: Location<'context>, emit: impl FnOnce(&mut Self) -> R) -> R {
        let enclosing = std::mem::replace(
            &mut self.contract.source_unit.mlir.current_location,
            location,
        );
        let result = emit(self);
        self.contract.source_unit.mlir.current_location = enclosing;
        result
    }
}

impl<'contract, 'source_unit, 'context> Deref for FunctionScope<'contract, 'source_unit, 'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.contract.source_unit.mlir
    }
}
