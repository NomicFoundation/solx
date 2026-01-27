//!
//! The LLVM IR generator Solidity data.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::context::traits::solidity_data::ISolidityData;

///
/// The LLVM IR generator Solidity data.
///
/// Describes some data that is only relevant to Solidity.
///
#[derive(Debug, Default)]
pub struct SolidityData {
    /// The immutables identifier-to-offset mapping.
    /// If the runtime code is available and this field is set, `offsets` method below can return `None`
    /// for immutables that are never referenced in the runtime code,
    /// However, if it is unset and `immutables_dummy` is used, then `offsets` method will always return
    /// a set with a single offset to avoid stack-too-deep false negatives caused by missing immutable writing operations.
    immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
    /// The dummy mapping that is used in dummy compiler runs.
    /// For instance, when the runtime code with actual immutables is not available due to errors such as stack-too-deep,
    /// but we still want to try compiling the deploy code to check for other errors including stack-too-deep.
    /// In this case, `immutables` is `None`, and `immutables_dummy` is used to allocated the offsets of the immutables.
    immutables_dummy: BTreeMap<String, u64>,

    /// Current Solidity contract full path.
    contract_name: String,
    /// Current Solidity source code location.
    current_solc_location: Option<solx_utils::DebugInfoSolcLocation>,

    /// Solidity AST debug info.
    debug_info: Option<solx_utils::DebugInfo>,
}

impl ISolidityData for SolidityData {
    fn offsets(&mut self, id: &str) -> Option<BTreeSet<u64>> {
        match self.immutables.as_ref() {
            Some(immutables) => immutables.get(id).cloned(),
            None => {
                let mut offsets = BTreeSet::new();
                offsets.insert(self.get_or_allocate_dummy_immutable(id));
                Some(offsets)
            }
        }
    }

    fn debug_info_contract_definition(&self) -> Option<&solx_utils::DebugInfoContractDefinition> {
        self.debug_info
            .as_ref()?
            .contract_definitions
            .get(self.contract_name.as_str())
    }

    fn debug_info_function_definition(
        &self,
        node_id: usize,
    ) -> Option<&solx_utils::DebugInfoFunctionDefinition> {
        self.debug_info.as_ref()?.function_definitions.get(&node_id)
    }

    fn set_debug_info_solc_location(&mut self, solc_location: solx_utils::DebugInfoSolcLocation) {
        self.current_solc_location = Some(solc_location);
    }

    fn get_debug_info_solc_location(&self) -> Option<&solx_utils::DebugInfoSolcLocation> {
        self.current_solc_location.as_ref()
    }

    fn get_solx_location(&self) -> Option<&solx_utils::DebugInfoMappedLocation> {
        let debug_info = self.debug_info.as_ref()?;
        let solc_location = self.get_debug_info_solc_location()?;

        debug_info.ast_nodes.values().find_map(|ast_node| {
            if ast_node.solc_location.start == solc_location.start {
                Some(&ast_node.mapped_location)
            } else {
                None
            }
        })
    }
}

impl SolidityData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        contract_name: String,
        debug_info: Option<solx_utils::DebugInfo>,
    ) -> Self {
        Self {
            immutables,
            immutables_dummy: BTreeMap::new(),
            contract_name,
            current_solc_location: None,
            debug_info,
        }
    }

    ///
    /// Returns the current number of immutables values in the contract.
    ///
    pub fn immutables_dummy_size(&self) -> usize {
        self.immutables_dummy.len() * solx_utils::BYTE_LENGTH_FIELD
    }

    ///
    /// Allocates memory for an immutable value in the auxiliary heap.
    ///
    /// If the identifier is already known, just returns its offset.
    ///
    pub fn allocate_dummy_immutable(&mut self, identifier: &str) -> u64 {
        let number_of_elements = self.immutables_dummy.len();
        let new_offset = number_of_elements * solx_utils::BYTE_LENGTH_FIELD;
        *self
            .immutables_dummy
            .entry(identifier.to_owned())
            .or_insert(new_offset as u64)
    }

    ///
    /// Gets the offset of the immutable value.
    ///
    /// If the value is not yet allocated, then it is done forcibly.
    ///
    pub fn get_or_allocate_dummy_immutable(&mut self, identifier: &str) -> u64 {
        match self.immutables_dummy.get(identifier).copied() {
            Some(offset) => offset,
            None => self.allocate_dummy_immutable(identifier),
        }
    }
}
