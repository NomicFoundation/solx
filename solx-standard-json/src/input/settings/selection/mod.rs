//!
//! The `solc --standard-json` expected output selection.
//!

pub mod selector;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use self::selector::Selector;

///
/// The `solc --standard-json` expected output selection.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Selection {
    /// Inner selection map.
    #[serde(flatten)]
    inner: BTreeMap<String, BTreeMap<String, BTreeSet<Selector>>>,
}

impl Selection {
    /// Wildcard selection.
    pub const WILDCARD: &'static str = "*";

    /// Any contract selection, used for file-level AST.
    pub const ANY_CONTRACT: &'static str = "";

    ///
    /// A shortcut constructor.
    ///
    pub fn new(selectors: BTreeSet<Selector>) -> Self {
        let mut file_level = BTreeMap::new();
        let mut contract_level = BTreeMap::new();

        let mut per_file_selectors = BTreeSet::new();
        if selectors.contains(&Selector::AST) {
            per_file_selectors.insert(Selector::AST);
        }
        if selectors.contains(&Selector::Benchmarks) {
            per_file_selectors.insert(Selector::Benchmarks);
        }
        let mut per_contract_selectors = selectors;
        per_contract_selectors.remove(&Selector::AST);

        if !per_file_selectors.is_empty() {
            contract_level.insert(Self::ANY_CONTRACT.to_owned(), per_file_selectors);
        }
        if !per_contract_selectors.is_empty() {
            contract_level.insert(Self::WILDCARD.to_owned(), per_contract_selectors);
        }
        if !contract_level.is_empty() {
            file_level.insert(Self::WILDCARD.to_owned(), contract_level);
        }
        Self { inner: file_level }
    }

    ///
    /// Checks if the output element of the specified contract is selected.
    ///
    pub fn check_selection(&self, path: &str, name: Option<&str>, selector: Selector) -> bool {
        if let Some(file) = self.inner.get(Self::WILDCARD).or(self.inner.get(path)) {
            if let (Some(any), selector @ Selector::AST | selector @ Selector::Benchmarks) =
                (file.get(Self::ANY_CONTRACT).or(file.get(path)), selector)
            {
                return any.contains(&Selector::Any) || any.contains(&selector);
            }
            if let Some(contract) = file
                .get(Self::WILDCARD)
                .or(name.and_then(|name| file.get(name)))
            {
                match selector {
                    Selector::MethodIdentifiers
                    | Selector::EVMLegacyAssembly
                    | Selector::GasEstimates
                        if contract.contains(&Selector::EVM) =>
                    {
                        return true;
                    }
                    Selector::BytecodeObject
                    | Selector::BytecodeLLVMAssembly
                    | Selector::BytecodeOpcodes
                    | Selector::BytecodeLinkReferences
                    | Selector::BytecodeSourceMap
                    | Selector::BytecodeDebugInfo
                    | Selector::BytecodeFunctionDebugData
                    | Selector::BytecodeGeneratedSources
                        if contract.contains(&Selector::Bytecode)
                            || contract.contains(&Selector::EVM) =>
                    {
                        return true;
                    }
                    Selector::RuntimeBytecodeObject
                    | Selector::RuntimeBytecodeLLVMAssembly
                    | Selector::RuntimeBytecodeOpcodes
                    | Selector::RuntimeBytecodeLinkReferences
                    | Selector::RuntimeBytecodeImmutableReferences
                    | Selector::RuntimeBytecodeSourceMap
                    | Selector::RuntimeBytecodeDebugInfo
                    | Selector::RuntimeBytecodeFunctionDebugData
                    | Selector::RuntimeBytecodeGeneratedSources
                        if contract.contains(&Selector::RuntimeBytecode)
                            || contract.contains(&Selector::EVM) =>
                    {
                        return true;
                    }
                    selector
                        if contract.contains(&Selector::Any) || contract.contains(&selector) =>
                    {
                        return true;
                    }
                    _ => {}
                }
            }
        }
        false
    }

    ///
    /// Adds the specified selector to the output selection of all contracts.
    ///
    pub fn set_selector(&mut self, selector: Selector) {
        for file in self.inner.values_mut() {
            match selector {
                Selector::AST => {
                    file.entry(Self::ANY_CONTRACT.to_owned())
                        .or_default()
                        .insert(selector);
                }
                selector => {
                    for (name, contract) in file.iter_mut() {
                        if name == Self::ANY_CONTRACT {
                            continue;
                        }
                        contract.insert(selector);
                    }
                }
            }
        }
    }

    ///
    /// Normalizes the selection by converting multi-item selectors into single-item selectors.
    ///
    pub fn normalize(&mut self) {
        for file in self.inner.values_mut() {
            for contract in file.values_mut() {
                *contract = contract
                    .iter()
                    .flat_map(|selector| selector.into_single_selectors())
                    .collect::<BTreeSet<_>>();
            }
        }
    }

    ///
    /// Retains only the selectors that request data from `solc`.
    ///
    pub fn retain_solc(&mut self) {
        for file in self.inner.values_mut() {
            for contract in file.values_mut() {
                contract.retain(Selector::is_received_from_solc);
            }
        }
    }

    ///
    /// Checks if the bytecode is requested for at least one contract.
    ///
    pub fn is_bytecode_set_for_any(&self) -> bool {
        for file in self.inner.values() {
            for contract in file.values() {
                if contract.contains(&Selector::EVM)
                    || contract.contains(&Selector::Bytecode)
                    || contract.contains(&Selector::BytecodeObject)
                    || contract.contains(&Selector::RuntimeBytecode)
                    || contract.contains(&Selector::RuntimeBytecodeObject)
                    || contract.contains(&Selector::Any)
                {
                    return true;
                }
            }
        }
        false
    }

    ///
    /// Checks if the debug info is requested for at least one contract.
    ///
    pub fn is_debug_info_set_for_any(&self) -> bool {
        for file in self.inner.values() {
            for contract in file.values() {
                if contract.contains(&Selector::EVM)
                    || contract.contains(&Selector::BytecodeDebugInfo)
                    || contract.contains(&Selector::RuntimeBytecodeDebugInfo)
                {
                    return true;
                }
            }
        }
        false
    }

    ///
    /// Whether the selection is empty.
    ///
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    ///
    /// Creates the selection required for testing.
    ///
    /// Selects AST, bytecode, deployedBytecode, methodIdentifiers, and either Yul or EVMLegacyAssembly.
    ///
    pub fn new_required_for_testing(via_ir: bool) -> Self {
        let mut per_file_selectors = BTreeSet::new();
        per_file_selectors.insert(Selector::AST);

        let mut per_contract_selectors = BTreeSet::new();
        per_contract_selectors.insert(Selector::Bytecode);
        per_contract_selectors.insert(Selector::RuntimeBytecode);
        per_contract_selectors.insert(Selector::MethodIdentifiers);
        per_contract_selectors.insert(if via_ir {
            Selector::Yul
        } else {
            Selector::EVMLegacyAssembly
        });

        let mut file_level = BTreeMap::new();
        let mut contract_level = BTreeMap::new();
        contract_level.insert(Self::ANY_CONTRACT.to_owned(), per_file_selectors);
        contract_level.insert(Self::WILDCARD.to_owned(), per_contract_selectors);
        file_level.insert(Self::WILDCARD.to_owned(), contract_level);

        Self { inner: file_level }
    }
}
