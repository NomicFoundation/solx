//!
//! Public state-variable getter emission: slang's getter type is the signature, and the storage
//! walk supplies the steps and reads its body replays.
//!

pub mod leaf;
pub mod level;

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Function;
use solx_mlir::FunctionDispatch;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;
use solx_utils::DataLocation;

use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

use self::leaf::Leaf;
use self::level::Level;

/// The storage walk a getter body replays: the key and index levels peeled off the state
/// variable's declared type, and the read at the bottom.
pub struct Getter<'context> {
    /// The mapping and array levels, one per getter parameter.
    levels: Vec<Level<'context>>,
    /// The read at the bottom of the walk.
    leaf: Leaf<'context>,
}

impl<'context> Getter<'context> {
    /// Walks the declared type once: each mapping or array level consumes one getter parameter,
    /// and the type left at the bottom is the leaf.
    pub fn new(
        state_variable: &StateVariableDefinition,
        source_unit: &SourceUnitScope<'context>,
    ) -> Self {
        let mut current = state_variable
            .get_type()
            .expect("binder types every state variable");
        let mut location = None;
        let mut levels = Vec::new();
        let leaf = loop {
            match current {
                Type::Mapping(mapping_type) => {
                    let value_type = mapping_type.value_type();
                    let element_type =
                        source_unit.resolve(&value_type, Some(DataLocation::Storage));
                    levels.push(Level::Mapping {
                        entry_type: source_unit.pointer(
                            &value_type,
                            element_type,
                            DataLocation::Storage,
                        ),
                    });
                    location = Some(DataLocation::Storage);
                    current = value_type;
                }
                Type::Array(array_type) => {
                    levels.push(Level::Array);
                    current = array_type.element_type();
                }
                Type::FixedSizeArray(array_type) => {
                    levels.push(Level::Array);
                    current = array_type.element_type();
                }
                Type::Struct(struct_type) => {
                    let Definition::Struct(struct_definition) = struct_type.definition() else {
                        unreachable!("Slang StructType always references a Struct definition");
                    };
                    break Leaf::members(
                        &struct_definition,
                        &state_variable.getter_struct_members(),
                    );
                }
                other => break Leaf::Value(source_unit.resolve(&other, location)),
            }
        };
        Self { levels, leaf }
    }

    /// Replays the levels over the getter's entry arguments, then reads the leaf into the return
    /// values converted to `results`.
    pub fn returned_values(
        &self,
        mut place: Place<'context>,
        results: &[MlirType<'context>],
        entry: Block<'context>,
        context: &Context<'context>,
    ) -> Vec<Value<'context>> {
        for (index, level) in self.levels.iter().enumerate() {
            place = level.step(place, entry.argument(index), context);
        }
        self.leaf.read(place, results, context)
    }
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Emits the `sol.func` a `public` state variable declares, dispatched by its ABI selector
    /// with slang's getter type as its signature. A constant folds its initializer; a mutable
    /// variable reads its storage slot; an immutable loads its linked value.
    pub fn state_variable_getter(&mut self, state_variable: &StateVariableDefinition) {
        let selector = state_variable
            .compute_selector()
            .expect("a public state variable has a selector");
        let Some(Type::Function(getter_type)) = state_variable.getter_type() else {
            unreachable!("slang types a public state variable's getter as a function");
        };
        let signature = Function::new(
            SourceUnitScope::getter_symbol(state_variable),
            self.source_unit.function_type(&getter_type),
        );
        match state_variable.attributes().mutability() {
            StateVariableMutability::Constant => {
                let initializer = state_variable
                    .value()
                    .expect("a constant state variable is initialized");
                let entry = signature.define(
                    Some(selector),
                    FunctionDispatch::Getter,
                    StateMutability::Pure,
                    self,
                    self.contract.body,
                );
                self.function(entry, signature.function_type.results, |scope| {
                    let result_type = scope.return_types[0];
                    let value = scope.converted(&initializer, result_type);
                    scope.current_block().r#return(&[value], scope);
                });
            }
            StateVariableMutability::Mutable | StateVariableMutability::Transient => {
                let getter = Getter::new(state_variable, self.source_unit);
                let entry = signature.define(
                    Some(selector),
                    FunctionDispatch::Getter,
                    StateMutability::View,
                    self,
                    self.contract.body,
                );
                self.function(entry, signature.function_type.results, |scope| {
                    let (place, _) = scope.state_variable_place(state_variable);
                    let values =
                        getter.returned_values(place, &scope.return_types, entry.block, scope);
                    scope.current_block().r#return(&values, scope);
                });
            }
            StateVariableMutability::Immutable => {
                let entry = signature.define(
                    Some(selector),
                    FunctionDispatch::Getter,
                    StateMutability::View,
                    self,
                    self.contract.body,
                );
                self.function(entry, signature.function_type.results, |scope| {
                    let result_type = scope.return_types[0];
                    let element_type = scope.resolve_type(
                        &state_variable
                            .get_type()
                            .expect("binder types every state variable"),
                        None,
                    );
                    let value = Value::load_immutable(
                        &SourceUnitScope::state_variable_symbol(state_variable),
                        element_type,
                        scope,
                    )
                    .convert(result_type, scope);
                    scope.current_block().r#return(&[value], scope);
                });
            }
        }
    }
}

impl<'context> SourceUnitScope<'context> {
    /// The getter's symbol: its internal signature qualified by the node id, since internal
    /// signatures alone collide.
    pub fn getter_symbol(state_variable: &StateVariableDefinition) -> String {
        format!(
            "{}_{}",
            state_variable
                .compute_internal_signature()
                .expect("a public state variable has an internal signature"),
            state_variable.node_id(),
        )
    }
}
