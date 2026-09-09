//!
//! The constructor chain: the constructors a contract's creation runs, most-derived first, and the
//! argument lists that reach each of them.
//!

use std::collections::BTreeSet;
use std::collections::HashMap;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Block;
use solx_mlir::Function;
use solx_mlir::FunctionType;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

/// The constructor chain of the object being emitted: its own constructor, declared or
/// synthesized, at position 0, then every contract base of its linearisation that declares one.
/// Each constructor calls the next before its own body runs and evaluates the next one's argument
/// list at that call, so the bodies run most-base first and the lists evaluate in chain order, as
/// legacy codegen has them. A list may name the parameters of the constructor of the
/// contract providing it, so when the provider sits above the caller, its parameters ride as
/// trailing parameters of every constructor in between.
pub struct ConstructorChain {
    /// The object's own constructor, when the source declares one: position 0.
    head: Option<FunctionDefinition>,
    /// The constructors of the contract bases declaring one, in linearisation order: positions 1
    /// onward.
    bases: Vec<FunctionDefinition>,
    /// The argument list each base constructor receives, keyed by its position.
    arguments: HashMap<usize, Arguments>,
}

/// The argument list a base constructor receives, by where it is evaluated.
#[derive(Clone)]
enum Arguments {
    /// Provided by the constructor at `provider`, above the frame calling the target, whose
    /// parameters the list may name: they are bound from the values threaded down to the caller.
    Threaded {
        provider: usize,
        list: ArgumentsDeclaration,
    },
    /// Evaluated in the calling frame as it is: provided by the caller itself, or by a contract
    /// declaring no constructor.
    Local(ArgumentsDeclaration),
}

impl ConstructorChain {
    /// Walks `contracts`, the object's linearisation with itself first, for the constructors it
    /// declares and the argument lists it supplies. A library's linearisation holds no contract.
    pub fn new(contracts: &[ContractDefinition]) -> Self {
        let mut chain = Self {
            head: contracts.first().and_then(ContractDefinition::constructor),
            bases: contracts
                .iter()
                .skip(1)
                .filter_map(ContractDefinition::constructor)
                .collect(),
            arguments: HashMap::new(),
        };
        for providing in contracts {
            let constructor = providing.constructor();
            let provider = constructor.as_ref().map(|constructor| {
                chain
                    .position(constructor)
                    .expect("a linearised contract's constructor is in the chain")
            });
            let specifiers = providing.inheritance_types();
            let invocations = constructor
                .as_ref()
                .map(|constructor| constructor.attributes().modifier_invocations());
            let lists = specifiers
                .iter()
                .map(|specifier| {
                    (
                        specifier.type_name().resolve_to_definition(),
                        specifier.arguments(),
                    )
                })
                .chain(invocations.iter().flat_map(|invocations| {
                    invocations.iter().map(|invocation| {
                        (
                            invocation.name().resolve_to_definition(),
                            invocation.arguments(),
                        )
                    })
                }));
            for (base, list) in lists {
                if let Some(Definition::Contract(base)) = base
                    && let Some(list) = list
                    && let Some(constructor) = base.constructor()
                {
                    let target = chain.position(&constructor).expect(
                        "slang admits a base-constructor argument list for a contract outside the hierarchy",
                    );
                    let arguments = match provider {
                        Some(provider) if provider + 1 < target => {
                            Arguments::Threaded { provider, list }
                        }
                        _ => Arguments::Local(list),
                    };
                    chain.arguments.insert(target, arguments);
                }
            }
        }
        chain
    }

    /// The position of `function` in the chain: 0 for the object's own constructor, a later one
    /// for a base's, `None` for a function that is no constructor of the chain.
    pub fn position(&self, function: &FunctionDefinition) -> Option<usize> {
        self.constructors().position(|constructor| {
            constructor.is_some_and(|constructor| constructor.node_id() == function.node_id())
        })
    }

    /// The constructor at `position`: absent past the chain's end, and at position 0 when the
    /// object's is synthesized.
    pub fn constructor(&self, position: usize) -> Option<&FunctionDefinition> {
        self.constructors().nth(position).flatten()
    }

    /// The signature of the base constructor at `position`: its symbol and its declared parameters,
    /// followed by the parameters of each constructor threaded through it.
    pub fn signature<'context>(
        &self,
        position: usize,
        source_unit: &SourceUnitScope<'context>,
    ) -> Function<'context> {
        Function::new(
            SourceUnitScope::function_symbol(
                self.constructor(position)
                    .expect("a base constructor occupies every position past the head"),
            ),
            FunctionType {
                parameters: self
                    .layout(position)
                    .flat_map(|declaring| {
                        source_unit
                            .signature_type(
                                self.constructor(declaring)
                                    .expect("a threaded provider declares a constructor"),
                            )
                            .parameters
                    })
                    .collect(),
                results: Vec::new(),
            },
        )
    }

    /// The constructors whose parameters the frame at `position` carries, in operand order: its
    /// own, then each one threaded through it ascending.
    fn layout(&self, position: usize) -> impl Iterator<Item = usize> {
        std::iter::once(position).chain(self.threaded(position))
    }

    /// The positions of the constructors whose parameters the one at `position` receives as
    /// trailing parameters: those above it providing a list for a constructor below it.
    fn threaded(&self, position: usize) -> BTreeSet<usize> {
        self.arguments
            .iter()
            .filter_map(|(target, arguments)| match arguments {
                Arguments::Threaded { provider, .. }
                    if *provider < position && position < *target =>
                {
                    Some(*provider)
                }
                _ => None,
            })
            .collect()
    }

    /// The parameter values the frame at `position` carries in `entry`, keyed by the position of
    /// the constructor declaring them, as [`Self::layout`] lays them.
    fn parameters<'context>(
        &self,
        position: usize,
        entry: Block<'context>,
    ) -> HashMap<usize, Vec<Value<'context>>> {
        let mut parameters = HashMap::new();
        let mut offset = 0;
        for declaring in self.layout(position) {
            let count = self
                .constructor(declaring)
                .map_or(0, |constructor| constructor.parameters().len());
            parameters.insert(
                declaring,
                (offset..offset + count)
                    .map(|index| entry.argument(index))
                    .collect(),
            );
            offset += count;
        }
        parameters
    }

    /// The constructor at each position, the object's own absent when synthesized.
    fn constructors(&self) -> impl Iterator<Item = Option<&FunctionDefinition>> {
        std::iter::once(self.head.as_ref()).chain(self.bases.iter().map(Some))
    }
}

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Emits the call to the constructor following `position` in the chain, when one follows,
    /// defining it at this first naming. The callee's argument list is evaluated here, one naming
    /// the parameters of a constructor above this frame binding them from the values threaded into
    /// `entry`; the values of every constructor threaded through the callee follow the arguments. A
    /// constructor no contract provides a list for declares no parameter.
    pub fn base_constructor_call(&mut self, position: usize, entry: Block<'context>) {
        let target = position + 1;
        let Some(callee) = self.contract.chain.constructor(target).cloned() else {
            return;
        };
        self.contract.function_definition(&callee);
        let carried = self.contract.chain.parameters(position, entry);
        let callee_parameters = callee.parameters();
        let mut operands = match self.contract.chain.arguments.get(&target).cloned() {
            Some(Arguments::Threaded { provider, list }) => {
                let parameters = self
                    .contract
                    .chain
                    .constructor(provider)
                    .expect("a threaded list's provider declares a constructor")
                    .parameters();
                self.nested(|scope| {
                    scope.bind_parameters(&parameters, &carried[&provider]);
                    scope.arguments_declaration(&list, &callee_parameters)
                })
            }
            Some(Arguments::Local(list)) => self.arguments_declaration(&list, &callee_parameters),
            None => Vec::new(),
        };
        for provider in self.contract.chain.threaded(target) {
            operands.extend(&carried[&provider]);
        }
        let signature = self.contract.signature(&callee);
        Function::call(&signature, &operands, self);
    }
}
