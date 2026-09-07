//!
//! The constructor chain: the constructors a contract's creation runs, most-derived first, and the
//! argument lists that reach each of them.
//!

pub mod arguments;

use std::collections::BTreeSet;
use std::collections::HashMap;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocations;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Block;
use solx_mlir::Function;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

use self::arguments::Arguments;

/// The constructor chain of the object being emitted: its own constructor, declared or
/// synthesized, then every contract base of its linearisation that declares one.
/// Each constructor calls the next before its own body runs and evaluates the next one's argument
/// list at that call, so the bodies run most-base first and the lists evaluate in chain order, as
/// legacy codegen has them. A list may name the parameters of the constructor of the
/// contract providing it, so when the provider sits above the caller, its parameters ride as
/// trailing parameters of every constructor in between.
pub struct ConstructorChain {
    /// The constructors the creation runs, in call order: the object's own, absent when it
    /// declares none, then every base of its linearisation that declares one. A library runs none.
    pub constructors: Vec<Option<FunctionDefinition>>,
    /// The position of each constructor the chain runs, keyed by its definition id.
    pub positions: HashMap<NodeId, usize>,
    /// The argument list each base constructor receives, keyed by its position.
    pub arguments: HashMap<usize, Arguments>,
}

impl ConstructorChain {
    /// The most-derived contract's constructor: the object's own, which the creation dispatches
    /// and whose frame runs the state variable initializers.
    pub const MOST_DERIVED: usize = 0;

    /// Walks `linearisation`, the object's with itself first, for the constructors it declares and
    /// the argument lists it supplies. A library's linearisation holds no contract.
    pub fn new(linearisation: Vec<ContractDefinition>) -> Self {
        let mut contracts = linearisation.iter();
        let constructors: Vec<Option<FunctionDefinition>> = contracts
            .next()
            .map(ContractDefinition::constructor)
            .into_iter()
            .chain(
                contracts
                    .filter_map(ContractDefinition::constructor)
                    .map(Some),
            )
            .collect();

        let positions: HashMap<NodeId, usize> = constructors
            .iter()
            .enumerate()
            .filter_map(|(position, constructor)| Some((constructor.as_ref()?.node_id(), position)))
            .collect();

        let mut arguments = HashMap::new();
        for contract in linearisation.iter() {
            let constructor = contract.constructor();
            let provider = constructor
                .as_ref()
                .and_then(|constructor| positions.get(&constructor.node_id()).copied());
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
                    && let Some(constructor) = base.constructor()
                {
                    arguments.insert(
                        positions[&constructor.node_id()],
                        Arguments {
                            provider,
                            list: ArgumentsDeclaration::PositionalArguments(list),
                        },
                    );
                }
            }
        }

        Self {
            constructors,
            positions,
            arguments,
        }
    }

    /// The constructors whose parameters the frame at `position` carries, in operand order: its
    /// own, then ascending each one above it that provides a list for a constructor below it,
    /// whose parameters ride through this frame.
    pub fn parameter_positions(&self, position: usize) -> impl Iterator<Item = usize> {
        std::iter::once(position).chain(
            self.arguments
                .iter()
                .filter_map(|(target, arguments)| {
                    arguments
                        .provider
                        .filter(|provider| *provider < position && position < *target)
                })
                .collect::<BTreeSet<usize>>(),
        )
    }

    /// The parameter values the frame at `position` carries in `entry`, keyed by the position of
    /// the constructor declaring them, in operand order.
    fn parameters<'context>(
        &self,
        position: usize,
        entry: Block<'context>,
    ) -> HashMap<usize, Vec<Value<'context>>> {
        let mut index = 0;
        let mut parameters = HashMap::new();
        for parameter_position in self.parameter_positions(position) {
            let mut values = Vec::new();
            for constructor in self.constructors[parameter_position].iter() {
                for _ in constructor.parameters().iter() {
                    values.push(entry.argument(index));
                    index += 1;
                }
            }
            parameters.insert(parameter_position, values);
        }
        parameters
    }
}

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Emits the call to the constructor following `position` in the chain, when one follows,
    /// defining it at this first naming. The callee's argument list is evaluated here, one naming
    /// the parameters of a constructor above this frame binding them from the values threaded into
    /// `entry`; the values of every constructor threaded through the callee follow the arguments. A
    /// constructor no contract provides a list for declares no parameter.
    pub fn base_constructor_call(&mut self, position: usize, entry: Block<'context>) {
        let chain = self.contract.chain;
        let callee_position = position + 1;
        let Some(callee) = chain
            .constructors
            .get(callee_position)
            .and_then(Option::as_ref)
        else {
            return;
        };
        let signature = self.contract.function_definition(callee);

        let mut parameter_values = chain.parameters(position, entry);
        let arguments = match chain.arguments.get(&callee_position) {
            Some(Arguments { provider, list }) => self.nested(|scope| {
                if let Some(provider) = *provider
                    && provider != position
                {
                    for (identifier, &value) in chain.constructors[provider]
                        .as_ref()
                        .expect("a threaded list's provider declares a constructor")
                        .parameters()
                        .iter()
                        .zip(&parameter_values[&provider])
                        .filter_map(|(parameter, value)| Some((parameter.name()?, value)))
                    {
                        scope.define_local(identifier.name(), value.r#type(), |_scope| value);
                    }
                }
                scope
                    .arguments_declaration(list, &callee.parameters())
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect()
            }),
            None => Vec::new(),
        };
        parameter_values.insert(callee_position, arguments);

        let operands: Vec<Value> = chain
            .parameter_positions(callee_position)
            .flat_map(|parameter_position| parameter_values[&parameter_position].iter().copied())
            .collect();
        Function::call(&signature, &operands, self);
    }
}
