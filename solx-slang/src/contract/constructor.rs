//!
//! Creation code emission: the initializer and the constructor chain in Slang's linearisation
//! order.
//!

use std::collections::HashMap;
use std::vec::IntoIter;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocations;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::PositionalArguments;

use solx_mlir::Block;
use solx_mlir::Function;
use solx_mlir::FunctionDispatch;
use solx_mlir::FunctionKind;
use solx_mlir::FunctionOrigin;
use solx_mlir::FunctionType;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::contract::ContractScope;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

/// Mutable state shared while emitting the object's constructor chain.
///
/// Constructor calls follow Slang's linearisation from derived to base. Each constructor
/// calls the next before executing its body, so bodies execute from base to derived.
///
/// Arguments supplied for a later base travel through intermediate constructors as extra
/// MLIR parameters. For a chain Leaf → Middle → Root, where Leaf supplies
/// `Middle(seed + 1)` and `Root(seed)`, Middle receives its declared argument and an extra
/// parameter carrying `seed`, which it passes to Root.
///
/// As in legacy solc, the creation entry point, declared or synthesized, evaluates every base
/// constructor's argument list before any body runs, in linearisation order whichever
/// constructor supplies it. A list names the parameters of the constructor that supplies it,
/// so each base's evaluated arguments live in stack slots that later lists read and write
/// under those names. On entry, the forwarded values are rebound to the new block's arguments.
pub struct ConstructorBuilder<'context> {
    /// The contracts of the object's linearisation, consumed in call order.
    pub contracts: IntoIter<ContractDefinition>,
    /// Base constructor argument lists by the constructor each is for.
    pub arguments: HashMap<NodeId, Arguments>,
    /// The base constructors still to be called, in call order, each with its evaluated
    /// arguments, which the constructors before it carry as extra parameters.
    pub forwarded: Vec<(FunctionDefinition, Vec<Value<'context>>)>,
}

/// The argument list a base constructor receives.
pub struct Arguments {
    /// The constructor that supplies the list, whose parameters it may name, if its contract
    /// declares one.
    pub provider: Option<NodeId>,
    /// The argument list as written.
    pub arguments: PositionalArguments,
}

impl<'context> ConstructorBuilder<'context> {
    /// Creates a constructor sequence for the contracts of an object's linearisation.
    pub fn new(contracts: Vec<ContractDefinition>) -> Self {
        Self {
            contracts: contracts.into_iter(),
            arguments: HashMap::new(),
            forwarded: Vec::new(),
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
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Emits the function that runs the state variable initializers, which the creation code
    /// calls ahead of the constructor's checks, as legacy runs the initializers first.
    pub fn initializer(&mut self) {
        let signature = Function::new("@initializer()".to_owned(), FunctionType::default());
        let entry = signature.define(
            "initializer",
            None,
            FunctionDispatch::Kind(FunctionKind::Initializer),
            FunctionOrigin::Synthesized,
            StateMutability::NonPayable,
            self,
            self.contract.body,
        );
        self.function(entry, true, &signature, |scope| {
            scope.state_variable_initializers();
            scope.current_block().r#return(&[], scope);
        });
    }

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
            SourceUnitScope::CONSTRUCTOR_KEYWORD,
            None,
            FunctionDispatch::Kind(FunctionKind::Constructor),
            FunctionOrigin::Synthesized,
            StateMutability::NonPayable,
            self,
            self.contract.body,
        );

        self.function(entry, true, &Function::constructor(), |scope| {
            scope.base_constructor_arguments();
            scope.base_constructor_call();
            scope.current_block().r#return(&[], scope);
        });
    }

    /// Collects the base constructor argument lists supplied by `contract`.
    fn inheritance_arguments(&mut self, contract: &ContractDefinition) {
        let constructor = contract.constructor();
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
                self.constructor.arguments.insert(
                    function.node_id(),
                    Arguments {
                        provider: constructor.as_ref().map(FunctionDefinition::node_id),
                        arguments: list,
                    },
                );
            }
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
    /// Evaluates every base constructor's argument list into stack slots, which are read only
    /// once every list has run. A list runs with its provider's parameters bound to the
    /// provider's slots. The conversions and slots of a base's arguments carry that base
    /// constructor's location, since they are part of running it.
    pub fn base_constructor_arguments(&mut self) {
        let mut chain: Vec<(
            FunctionDefinition,
            Vec<(Place<'context>, MlirType<'context>)>,
        )> = Vec::new();
        while let Some(constructor) = self.contract.next_constructor() {
            let signature = self.contract.source_unit.function_signature(&constructor);
            let slots = self
                .contract
                .constructor
                .arguments
                .remove(&constructor.node_id())
                .map(|list| {
                    let arguments: Vec<_> = list.arguments.iter().collect();
                    self.nested(|scope| {
                        if let Some((provider, slots)) = chain
                            .iter()
                            .find(|(base, _)| Some(base.node_id()) == list.provider)
                        {
                            for (parameter, &(place, element_type)) in
                                provider.parameters().iter().zip(slots)
                            {
                                if let Some(identifier) = parameter.name() {
                                    scope.environment.define_variable(
                                        identifier.name(),
                                        place,
                                        element_type,
                                    );
                                }
                            }
                        }
                        scope.at_node(&constructor, |scope| {
                            scope
                                .converted_arguments(
                                    &arguments,
                                    &signature.function_type.parameters,
                                )
                                .into_iter()
                                .map(|value| {
                                    let place = Place::stack(value.r#type(), scope);
                                    place.store(value, scope);
                                    (place, value.r#type())
                                })
                                .collect()
                        })
                    })
                })
                .unwrap_or_default();
            chain.push((constructor, slots));
        }
        for (constructor, slots) in chain {
            let values = self.at_node(&constructor, |scope| {
                slots
                    .into_iter()
                    .map(|(place, element_type)| place.load(element_type, scope))
                    .collect()
            });
            self.contract
                .constructor
                .forwarded
                .push((constructor, values));
        }
    }

    /// Calls the next constructor of the chain with its arguments and forwards the ones for the
    /// bases below it. The call carries that constructor's location.
    pub fn base_constructor_call(&mut self) {
        if self.contract.constructor.forwarded.is_empty() {
            return;
        }
        let (callee, mut operands) = self.contract.constructor.forwarded.remove(0);
        operands.extend(
            self.contract
                .constructor
                .forwarded
                .iter()
                .flat_map(|(_, values)| values.iter().copied()),
        );
        let signature = self.contract.function_definition(&callee);
        self.at_node(&callee, |scope| {
            Function::call(&signature, &operands, scope)
        });
    }
}
