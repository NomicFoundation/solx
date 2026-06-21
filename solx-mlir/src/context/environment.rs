//!
//! Variable environment for MLIR code generation.
//!

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Parameter;

use crate::Builder;
use crate::Pointer;
use crate::Type;

/// Tracks variable places (alloca'd pointers) for lexical scoping.
///
/// Each variable maps to the place holding it — a `!sol.ptr<T, Stack>` for a
/// Solidity local, parameter, or named return, an `!llvm.ptr` for a Yul
/// inline-assembly local. A Solidity read reconstructs the [`Pointer`] from the
/// place and loads its `pointee()`; a Yul read reinterprets the place to an
/// `!llvm.ptr`. The element type is the place's own pointee, so it is not stored
/// separately.
///
/// [`Pointer`]: crate::Pointer
///
/// Bindings are keyed by the declaration's Slang [`NodeId`], not its textual
/// name, so same-named locals across scopes (shadowing) are distinct by
/// construction and identifier references resolve through the binder
/// (`resolve_to_definition().node_id()`) rather than by name. Lexical scoping is
/// still tracked: lookups search from the innermost scope outward, and
/// `enter_scope()` / `exit_scope()` bracket blocks that introduce new variables.
pub struct Environment<'context, 'block> {
    /// Stack of scopes, each mapping a declaration's [`NodeId`] to its place.
    /// The outermost scope (index 0) holds function parameters.
    scopes: Vec<HashMap<NodeId, Value<'context, 'block>>>,
}

impl<'context, 'block> Default for Environment<'context, 'block> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'context, 'block> Environment<'context, 'block> {
    /// Creates a new environment with a single root scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Pushes a new lexical scope.
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pops the innermost lexical scope.
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Registers a variable, keyed by its declaration's [`NodeId`], with its
    /// place (alloca'd pointer) in the current scope.
    pub fn define_variable(&mut self, declaration: NodeId, pointer: Value<'context, 'block>) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(declaration, pointer);
    }

    /// Binds `parameter` to `value` in the current scope: coerces the value to the
    /// parameter's declared type (an untyped catch binding defaults to `ui256`),
    /// spills it to a fresh stack slot, and defines the parameter by node id.
    /// Shared by the `try` success returns and the `catch` clause payload.
    pub fn bind_parameter(
        &mut self,
        parameter: &Parameter,
        value: Value<'context, 'block>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        let parameter_type = Type::parameter(parameter, builder);
        let cast = crate::Value::new(value).cast(Type::new(parameter_type), builder, block);
        let pointer = Pointer::stack_slot(Type::new(parameter_type), builder, block);
        pointer.store(cast, builder, block);
        self.define_variable(parameter.node_id(), pointer.into_mlir());
    }

    /// Binds each of `function`'s parameters from its incoming entry-block
    /// argument: a fresh stack slot holding the argument, defined by node id. The
    /// block arguments already carry the parameter types, so — unlike
    /// [`Self::bind_parameter`] — no coercion is needed.
    pub fn bind_parameters(
        &mut self,
        function: &FunctionDefinition,
        parameter_types: &[MlirType<'context>],
        entry_block: &BlockRef<'context, 'block>,
        builder: &Builder<'context>,
    ) {
        for (index, parameter) in function.parameters().iter().enumerate() {
            self.bind_block_argument(
                parameter.node_id(),
                parameter_types[index],
                index,
                entry_block,
                builder,
            );
        }
    }

    /// Spills the entry block's argument at `argument_index` into a fresh stack
    /// slot of `mlir_type` and binds `declaration` to it. The block argument
    /// already carries the type, so no coercion is needed. The atomic binding
    /// [`Self::bind_parameters`] and a modifier-stage func are each built from.
    pub fn bind_block_argument(
        &mut self,
        declaration: NodeId,
        mlir_type: MlirType<'context>,
        argument_index: usize,
        entry_block: &BlockRef<'context, 'block>,
        builder: &Builder<'context>,
    ) {
        let pointer =
            Pointer::from_argument(Type::new(mlir_type), argument_index, entry_block, builder);
        self.define_variable(declaration, pointer.into_mlir());
    }

    /// Allocates and binds a stack slot for each named return value, pushing
    /// `None` for an unnamed return. A `modifier_body` seeds every slot (named or
    /// not) from the trailing block arguments at the `parameter_count` offset, so
    /// the shared return state survives an empty body or a partial `_` reach;
    /// otherwise only the named slots are default-initialised. Returns the slot
    /// places, parallel to the returns.
    pub fn initialize_return_slots(
        &mut self,
        function: &FunctionDefinition,
        result_types: &[MlirType<'context>],
        parameter_count: usize,
        modifier_body: bool,
        entry_block: &BlockRef<'context, 'block>,
        builder: &Builder<'context>,
    ) -> Vec<Option<Value<'context, 'block>>> {
        let mut return_slots: Vec<Option<Value<'context, 'block>>> = Vec::new();
        let Some(returns) = function.returns() else {
            return return_slots;
        };
        for (index, parameter) in returns.iter().enumerate() {
            let return_type = Type::new(result_types[index]);
            if modifier_body {
                let pointer = Pointer::from_argument(
                    return_type,
                    parameter_count + index,
                    entry_block,
                    builder,
                );
                if parameter.name().is_some() {
                    self.define_variable(parameter.node_id(), pointer.into_mlir());
                }
                return_slots.push(Some(pointer.into_mlir()));
            } else if parameter.name().is_none() {
                return_slots.push(None);
            } else {
                let pointer =
                    Pointer::default_initialized(return_type, builder, entry_block).into_mlir();
                self.define_variable(parameter.node_id(), pointer);
                return_slots.push(Some(pointer));
            }
        }
        return_slots
    }

    /// Looks up a variable's place by its declaration's [`NodeId`] (from
    /// `resolve_to_definition().node_id()`).
    ///
    /// Searches from the innermost scope outward.
    pub fn variable(&self, declaration: NodeId) -> Value<'context, 'block> {
        for scope in self.scopes.iter().rev() {
            if let Some(pointer) = scope.get(&declaration) {
                return *pointer;
            }
        }
        unreachable!("unregistered local variable: {declaration:?}");
    }
}
