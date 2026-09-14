//!
//! The JSON ABI of a contract, interface or library in solc's spelling. Slang computes every
//! entry, its order and its getter flattening; this module only renders them.
//!

use std::collections::BTreeMap;

use serde::Serialize;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::abi::AbiMutability;
use slang_solidity_v2::abi::AbiParameter;
use slang_solidity_v2::abi::AbiType;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::InterfaceDefinition;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::StateVariableDefinition;

/// The `abi` field of a standard-JSON contract: the entries in Slang's order, which is solc's.
///
/// `serde_json::to_value` sorts object keys, so the rendered JSON carries solc's alphabetical
/// key order regardless of field declaration order here.
#[derive(Debug, Serialize)]
pub struct Abi(Vec<Entry>);

impl Abi {
    /// The value the standard-JSON output stores.
    pub fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("the ABI only holds strings, booleans and lists")
    }

    /// Composes the ABI of a definition Slang has no whole ABI for from the entries it computes
    /// per member, in Slang's order.
    fn from_members(members: impl Iterator<Item = ContractMember>) -> Self {
        let mut entries = members
            .filter_map(|member| match member {
                ContractMember::FunctionDefinition(function) => function.compute_abi_entry(),
                ContractMember::ErrorDefinition(error) => error.compute_abi_entry(),
                ContractMember::EventDefinition(event) => event.compute_abi_entry(),
                _ => None,
            })
            .collect::<Vec<AbiEntry>>();
        entries.sort();
        Self(entries.iter().map(Entry::from).collect())
    }
}

impl From<&ContractDefinition> for Abi {
    fn from(contract: &ContractDefinition) -> Self {
        let abi = contract
            .compute_abi()
            .expect("slang admits a contract whose ABI it cannot compute");
        Self(abi.entries().iter().map(Entry::from).collect())
    }
}

/// An interface's own members; Slang does not expose an interface's linearisation, so members
/// inherited from base interfaces are not listed.
impl From<&InterfaceDefinition> for Abi {
    fn from(interface: &InterfaceDefinition) -> Self {
        Self::from_members(interface.members().iter())
    }
}

impl From<&LibraryDefinition> for Abi {
    fn from(library: &LibraryDefinition) -> Self {
        Self::from_members(library.members().iter())
    }
}

/// The `evm.methodIdentifiers` map: each externally dispatchable function keyed by the signature
/// its selector hashes, each public state variable by its canonical one, selectors in lower-case
/// hex.
pub struct MethodIdentifiers(BTreeMap<String, String>);

impl MethodIdentifiers {
    /// Maps the given functions and state variables; a function that is not a regular external
    /// one, or a state variable that is not public, has no identifier and is skipped.
    pub fn new(
        functions: impl IntoIterator<Item = FunctionDefinition>,
        state_variables: impl IntoIterator<Item = StateVariableDefinition>,
    ) -> Self {
        Self(
            functions
                .into_iter()
                .filter(|function| {
                    matches!(function.kind(), FunctionKind::Regular)
                        && function.is_externally_visible()
                })
                .map(|function| {
                    (
                        function
                            .compute_selector_signature()
                            .expect("an externally visible function has a selector signature"),
                        function
                            .compute_selector()
                            .expect("an externally visible function has a selector"),
                    )
                })
                .chain(
                    state_variables
                        .into_iter()
                        .filter(StateVariableDefinition::is_externally_visible)
                        .map(|state_variable| {
                            (
                                state_variable
                                    .compute_canonical_signature()
                                    .expect("a public state variable has a canonical signature"),
                                state_variable
                                    .compute_selector()
                                    .expect("a public state variable has a selector"),
                            )
                        }),
                )
                .map(|(signature, selector)| (signature, format!("{selector:08x}")))
                .collect(),
        )
    }

    /// The map the standard-JSON output stores.
    pub fn into_map(self) -> BTreeMap<String, String> {
        self.0
    }
}

/// The whole hierarchy, as the ABI lists it.
impl From<&ContractDefinition> for MethodIdentifiers {
    fn from(contract: &ContractDefinition) -> Self {
        Self::new(
            contract.linearised_functions(),
            contract.linearised_state_variables(),
        )
    }
}

impl From<&InterfaceDefinition> for MethodIdentifiers {
    fn from(interface: &InterfaceDefinition) -> Self {
        Self::new(
            interface
                .members()
                .iter()
                .filter_map(|member| match member {
                    ContractMember::FunctionDefinition(function) => Some(function),
                    _ => None,
                }),
            std::iter::empty(),
        )
    }
}

/// One ABI entry, tagged by `type`.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Entry {
    /// The constructor, nameless and without outputs.
    Constructor {
        /// The constructor parameters.
        inputs: Vec<Parameter>,
        /// `nonpayable` or `payable`.
        #[serde(rename = "stateMutability")]
        state_mutability: Mutability,
    },
    /// A custom error.
    Error {
        /// The error parameters.
        inputs: Vec<Parameter>,
        /// The error name.
        name: String,
    },
    /// An event.
    Event {
        /// Whether the event is declared `anonymous`.
        anonymous: bool,
        /// The event parameters, each carrying `indexed`.
        inputs: Vec<Parameter>,
        /// The event name.
        name: String,
    },
    /// The fallback function.
    Fallback {
        /// `nonpayable` or `payable`.
        #[serde(rename = "stateMutability")]
        state_mutability: Mutability,
    },
    /// An externally visible function or a public state variable's getter.
    Function {
        /// The function parameters.
        inputs: Vec<Parameter>,
        /// The function name.
        name: String,
        /// The return values; a struct-returning getter is flattened to its members by Slang.
        outputs: Vec<Parameter>,
        /// The declared mutability.
        #[serde(rename = "stateMutability")]
        state_mutability: Mutability,
    },
    /// The receive function.
    Receive {
        /// Always `payable`.
        #[serde(rename = "stateMutability")]
        state_mutability: Mutability,
    },
}

impl From<&AbiEntry> for Entry {
    fn from(entry: &AbiEntry) -> Self {
        match entry {
            AbiEntry::Constructor(constructor) => Self::Constructor {
                inputs: Parameter::list(constructor.inputs()),
                state_mutability: constructor.state_mutability().into(),
            },
            AbiEntry::Error(error) => Self::Error {
                inputs: Parameter::list(error.inputs()),
                name: error.name().to_owned(),
            },
            AbiEntry::Event(event) => Self::Event {
                anonymous: event.anonymous(),
                inputs: event
                    .inputs()
                    .iter()
                    .map(|input| Parameter::from(input).indexed(input.indexed()))
                    .collect(),
                name: event.name().to_owned(),
            },
            AbiEntry::Fallback(fallback) => Self::Fallback {
                state_mutability: fallback.state_mutability().into(),
            },
            AbiEntry::Function(function) => Self::Function {
                inputs: Parameter::list(function.inputs()),
                name: function.name().to_owned(),
                outputs: Parameter::list(function.outputs()),
                state_mutability: function.state_mutability().into(),
            },
            AbiEntry::Receive(receive) => Self::Receive {
                state_mutability: receive.state_mutability().into(),
            },
        }
    }
}

/// A parameter, return value or tuple component.
#[derive(Debug, Serialize)]
struct Parameter {
    /// The members of a struct type, present only when `type` spells a tuple.
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<Parameter>>,
    /// Whether an event parameter is `indexed`; absent everywhere but event inputs, as in solc.
    #[serde(skip_serializing_if = "Option::is_none")]
    indexed: Option<bool>,
    /// The declared name, empty when unnamed or when Slang does not carry it (getters).
    name: String,
    /// The ABI type: `tuple` with array suffixes for structs, the canonical name otherwise.
    r#type: String,
}

impl Parameter {
    /// Renders a parameter list.
    fn list(parameters: &[AbiParameter]) -> Vec<Self> {
        parameters.iter().map(Self::from).collect()
    }

    /// Renders a named type; the recursion point for struct components.
    fn new(name: String, abi_type: &AbiType) -> Self {
        let (r#type, components) = Self::spell(abi_type);
        Self {
            components,
            indexed: None,
            name,
            r#type,
        }
    }

    /// Marks an event input.
    fn indexed(mut self, indexed: bool) -> Self {
        self.indexed = Some(indexed);
        self
    }

    /// Spells the `type` string and the components solc attaches to it. Slang's `Display` spells
    /// a struct in canonical-signature form `(T1,T2)`; solc's JSON spells it `tuple` and lists the
    /// members under `components`, keeping any array suffixes on the `tuple` word.
    fn spell(abi_type: &AbiType) -> (String, Option<Vec<Self>>) {
        match abi_type {
            AbiType::Array { element } => {
                let (element, components) = Self::spell(element);
                (format!("{element}[]"), components)
            }
            AbiType::FixedSizeArray { element, size } => {
                let (element, components) = Self::spell(element);
                (format!("{element}[{size}]"), components)
            }
            AbiType::Tuple(components) => (
                "tuple".to_owned(),
                Some(
                    components
                        .iter()
                        .map(|component| {
                            Self::new(component.name().to_owned(), component.abi_type())
                        })
                        .collect(),
                ),
            ),
            scalar => (scalar.to_string(), None),
        }
    }
}

impl From<&AbiParameter> for Parameter {
    fn from(parameter: &AbiParameter) -> Self {
        Self::new(
            parameter.name().unwrap_or_default().to_owned(),
            parameter.abi_type(),
        )
    }
}

/// The `stateMutability` spelling.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Mutability {
    /// Reads nothing.
    Pure,
    /// Reads state.
    View,
    /// Writes state, rejects value.
    NonPayable,
    /// Accepts value.
    Payable,
}

impl From<&AbiMutability> for Mutability {
    fn from(mutability: &AbiMutability) -> Self {
        match mutability {
            AbiMutability::Pure => Self::Pure,
            AbiMutability::View => Self::View,
            AbiMutability::NonPayable => Self::NonPayable,
            AbiMutability::Payable => Self::Payable,
        }
    }
}
