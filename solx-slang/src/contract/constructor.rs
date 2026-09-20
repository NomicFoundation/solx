//!
//! Constructor emission in Slang's linearisation order.
//!

use std::collections::HashMap;
use std::vec::IntoIter;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocations;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Block;
use solx_mlir::Function;
use solx_mlir::FunctionDispatch;
use solx_mlir::FunctionKind;
use solx_mlir::StateMutability;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::contract::ContractScope;
use crate::scope::function::FunctionScope;

/// The constructors and arguments emitted for the object's creation.
pub struct Constructor<'context> {
    /// The contracts of the object's linearisation, consumed in call order.
    pub contracts: IntoIter<ContractDefinition>,
    /// The argument lists supplied for bases, keyed by the provider's definition id.
    pub arguments: HashMap<Option<NodeId>, Vec<Arguments>>,
    /// The evaluated arguments awaiting delivery to base constructors, in linearisation order.
    pub forwarded: Vec<(NodeId, Vec<Value<'context>>)>,
    /// The linearisation index of each contract by definition id.
    pub positions: HashMap<NodeId, usize>,
    /// The constructor definition id currently being emitted.
    pub current: Option<NodeId>,
}

/// The argument list a base constructor receives.
pub struct Arguments {
    /// The base constructor that receives the arguments.
    pub function: FunctionDefinition,
    /// The argument list as written.
    pub arguments: ArgumentsDeclaration,
}

impl<'context> Constructor<'context> {
    /// Creates a constructor sequence for the contracts of an object's linearisation.
    pub fn new(contracts: Vec<ContractDefinition>) -> Self {
        let positions = contracts
            .iter()
            .enumerate()
            .map(|(index, contract)| (contract.node_id(), index))
            .collect();
        Self {
            contracts: contracts.into_iter(),
            arguments: HashMap::new(),
            forwarded: Vec::new(),
            positions,
            current: None,
        }
    }

    /// The MLIR types of the forwarded base constructor arguments.
    pub fn parameter_types(&self) -> impl Iterator<Item = MlirType<'context>> + '_ {
        self.forwarded
            .iter()
            .flat_map(|(_, values)| values.iter().map(|value| value.r#type()))
    }

    /// Binds incoming forwarded arguments from the entry block.
    pub fn bind_parameters(&mut self, function: &FunctionDefinition, entry: Block<'context>) {
        let mut index = function.parameters().iter().count();
        for (_, values) in &mut self.forwarded {
            for value in values {
                *value = entry.argument(index);
                index += 1;
            }
        }
    }

    /// Extracts the evaluated arguments destined for `target`.
    pub fn take_arguments(&mut self, target: NodeId) -> Vec<Value<'context>> {
        if let Some(position) = self
            .forwarded
            .iter()
            .position(|(node_id, _)| *node_id == target)
        {
            self.forwarded.remove(position).1
        } else {
            Vec::new()
        }
    }
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Emits the creation entry point, synthesizing one when the contract declares none.
    pub fn constructor(&mut self) {
        let Some(contract) = self.next_contract() else {
            return;
        };
        if let Some(constructor) = contract.constructor() {
            self.function_definition(&constructor);
            return;
        }

        let entry = Function::constructor().define(
            None,
            FunctionDispatch::Kind(FunctionKind::Constructor),
            StateMutability::NonPayable,
            self,
            self.contract.body,
        );

        self.constructor.current = None;
        self.function(entry, true, &Function::constructor(), |scope| {
            scope.state_variable_initializers();
            scope.base_constructor_call();
            scope.current_block().r#return(&[], scope);
        });
    }

    /// Collects the base constructor argument lists supplied by `contract` in linearisation order.
    fn inheritance_arguments(&mut self, contract: &ContractDefinition) {
        let constructor = contract.constructor();
        let mut arguments = Vec::new();
        for (base, list) in contract
            .inheritance_types()
            .iter()
            .map(|specifier| {
                (
                    specifier.type_name().resolve_to_definition(),
                    specifier.arguments(),
                )
            })
            .chain(
                constructor
                    .as_ref()
                    .map(|constructor| constructor.attributes().modifier_invocations())
                    .iter()
                    .flat_map(ModifierInvocations::iter)
                    .map(|invocation| {
                        (
                            invocation.name().resolve_to_definition(),
                            invocation.arguments(),
                        )
                    }),
            )
        {
            if let Some(Definition::Contract(base)) = base
                && let Some(list) = list
                && let Some(function) = base.constructor()
            {
                arguments.push((
                    self.constructor
                        .positions
                        .get(&base.node_id())
                        .copied()
                        .unwrap_or_default(),
                    Arguments {
                        function,
                        arguments: ArgumentsDeclaration::PositionalArguments(list),
                    },
                ));
            }
        }
        if !arguments.is_empty() {
            arguments.sort_by_key(|(position, _)| *position);
            let provider = constructor.as_ref().map(FunctionDefinition::node_id);
            self.constructor
                .arguments
                .entry(provider)
                .or_default()
                .extend(arguments.into_iter().map(|(_, argument)| argument));
        }
    }

    /// Reaches the next declared constructor, retaining arguments supplied by intervening bases.
    fn next_constructor(&mut self) -> Option<FunctionDefinition> {
        while let Some(contract) = self.next_contract() {
            if let Some(constructor) = contract.constructor() {
                return Some(constructor);
            }
        }
        None
    }

    /// Advances to the next contract in linearisation order, collecting its inheritance arguments.
    fn next_contract(&mut self) -> Option<ContractDefinition> {
        let contract = self.constructor.contracts.next()?;
        self.inheritance_arguments(&contract);
        Some(contract)
    }
}

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Evaluates base constructor arguments provided here, forwards arguments needed by bases
    /// below, and emits the call to the next constructor in the linearisation.
    pub fn base_constructor_call(&mut self) {
        if let Some(arguments) = self
            .contract
            .constructor
            .arguments
            .remove(&self.contract.constructor.current)
        {
            for argument in arguments {
                let values: Vec<Value<'context>> = self
                    .arguments_declaration(&argument.arguments, &argument.function.parameters())
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect();
                self.contract
                    .constructor
                    .forwarded
                    .push((argument.function.node_id(), values));
            }
        }

        let Some(callee) = self.contract.next_constructor() else {
            return;
        };

        if let Some(arguments) = self.contract.constructor.arguments.remove(&None) {
            for argument in arguments {
                let values: Vec<Value<'context>> = self
                    .arguments_declaration(&argument.arguments, &argument.function.parameters())
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect();
                self.contract
                    .constructor
                    .forwarded
                    .push((argument.function.node_id(), values));
            }
        }

        let mut operands = self.contract.constructor.take_arguments(callee.node_id());
        operands.extend(
            self.contract
                .constructor
                .forwarded
                .iter()
                .flat_map(|(_, values)| values.iter().copied()),
        );
        let signature = self.contract.function_definition(&callee);
        Function::call(&signature, &operands, self);
    }
}
