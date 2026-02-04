//!
//! The Matter Labs compiler test.
//!

pub mod metadata;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use crate::compilers::Compiler;
use crate::compilers::mode::Mode;
use crate::directories::Buildable;
use crate::filters::Filters;
use crate::revm::address_iterator::AddressIterator;
use crate::summary::Summary;
use crate::test::Test;
use crate::test::case::Case;
use crate::test::description::TestDescription;
use crate::test::selector::TestSelector;

use self::metadata::Metadata;

/// The default simple contract name.
pub const SIMPLE_TESTS_CONTRACT_NAME: &str = "Test";

/// The default simple contract instance name.
pub const SIMPLE_TESTS_INSTANCE: &str = "Test";

/// The default address of the caller.
pub const DEFAULT_CALLER_ADDRESS: &str = "0xdeadbeef00000000000000000000000000000001";

///
/// Used for default initialization.
///
pub fn simple_tests_instance() -> String {
    SIMPLE_TESTS_INSTANCE.to_string()
}

///
/// Used for default initialization.
///
pub fn default_caller_address() -> String {
    DEFAULT_CALLER_ADDRESS.to_string()
}

///
/// The Matter Labs compiler test.
///
#[derive(Debug)]
pub struct MatterLabsTest {
    /// The test path.
    path: PathBuf,
    /// The test selector.
    selector: TestSelector,
    /// The test metadata.
    metadata: Metadata,
    /// The test sources.
    sources: Vec<(String, String)>,
}

impl MatterLabsTest {
    ///
    /// Try to create new test.
    ///
    pub fn new(path: PathBuf, summary: Arc<Mutex<Summary>>, filters: &Filters) -> Option<Self> {
        let selector = TestSelector {
            path: crate::utils::path_to_string_normalized(path.as_path()),
            case: None,
            input: None,
        };

        if !filters.check_test_path(selector.path.to_string().as_str()) {
            return None;
        }

        let test_description = TestDescription::default_for(selector.clone());

        let main_file_string = match std::fs::read_to_string(path.as_path()) {
            Ok(data) => data,
            Err(error) => {
                Summary::invalid(summary, test_description, error);
                return None;
            }
        };

        let mut metadata = match Metadata::from_str(main_file_string.as_str())
            .map_err(|error| anyhow::anyhow!("Invalid metadata JSON: {error}"))
        {
            Ok(metadata) => metadata,
            Err(error) => {
                Summary::invalid(summary, test_description, error);
                return None;
            }
        };

        if metadata.ignore {
            Summary::ignored(summary, test_description);
            return None;
        }

        if !filters.check_group(&metadata.group) {
            return None;
        }

        let sources = if metadata.contracts.is_empty() {
            if path.ends_with("test.json") {
                vec![]
            } else {
                vec![(
                    crate::utils::path_to_string_normalized(path.as_path()),
                    main_file_string,
                )]
            }
        } else {
            let mut sources = HashMap::new();
            let mut paths = HashSet::with_capacity(metadata.contracts.len());
            for (_, path_string) in metadata.contracts.iter_mut() {
                let mut file_path = path.clone();
                file_path.pop();
                let mut path_string_split = path_string.split(':');
                let file_relative_path = path_string_split.next().expect("Always exists");
                let contract_name = path_string_split.next();
                file_path.push(file_relative_path);

                let file_path_unified =
                    crate::utils::path_to_string_normalized(file_path.as_path());
                *path_string = if let Some(contract_name) = contract_name {
                    format!("{file_path_unified}:{contract_name}")
                } else {
                    file_path_unified.clone()
                };
                paths.insert(file_path_unified);
            }

            let mut test_directory_path = path.clone();
            test_directory_path.pop();
            for entry in
                glob::glob(format!("{}/**/*.sol", test_directory_path.to_string_lossy()).as_str())
                    .expect("Always valid")
                    .filter_map(Result::ok)
            {
                paths.insert(crate::utils::path_to_string_normalized(entry.as_path()));
            }

            for path in paths.into_iter() {
                let source_code = match std::fs::read_to_string(path.as_str())
                    .map_err(|error| anyhow::anyhow!("Reading source file error: {error}"))
                {
                    Ok(source) => source,
                    Err(error) => {
                        Summary::invalid(summary, test_description, error);
                        return None;
                    }
                };
                sources.insert(path, source_code);
            }
            sources.into_iter().collect()
        };

        metadata.cases.retain(|case| {
            let selector_with_case = TestSelector {
                path: selector.path.clone(),
                case: Some(case.name.clone()),
                input: selector.input.clone(),
            };
            if case.ignore {
                Summary::ignored(
                    summary.clone(),
                    TestDescription::default_for(selector_with_case),
                );
                return false;
            }
            let case_name = selector_with_case.to_string();
            if !filters.check_case_path(&case_name) {
                return false;
            }
            true
        });

        Some(Self {
            path,
            selector,
            metadata,
            sources,
        })
    }

    ///
    /// Checks if the test is not filtered out.
    ///
    fn check_filters(&self, filters: &Filters, mode: &Mode) -> Option<()> {
        if !filters.check_mode(mode) {
            return None;
        }
        if let Some(filters) = self.metadata.modes.as_ref()
            && !mode.check_extended_filters(filters.as_slice())
        {
            return None;
        }
        if let Some(toolchains) = self.metadata.toolchains.as_ref()
            && !toolchains.iter().any(|t| t == mode.toolchain())
        {
            return None;
        }
        if !mode.check_pragmas(&self.sources) {
            return None;
        }
        Some(())
    }

    ///
    /// Adds the default contract to the list of contracts if it is empty.
    ///
    fn push_default_contract(
        &self,
        contracts: &mut BTreeMap<String, String>,
        is_multi_contract: bool,
    ) {
        if contracts.is_empty() {
            let contract_name = if is_multi_contract {
                format!("{}:{}", self.selector.path, SIMPLE_TESTS_CONTRACT_NAME)
            } else {
                self.selector.path.to_string()
            };
            contracts.insert(SIMPLE_TESTS_INSTANCE.to_owned(), contract_name);
        }
    }

    ///
    /// Returns library information.
    ///
    fn get_libraries(
        &self,
        address_iterator: &mut AddressIterator,
    ) -> (
        solx_utils::Libraries,
        BTreeMap<String, web3::types::Address>,
    ) {
        let mut libraries = BTreeMap::new();
        let mut library_addresses = BTreeMap::new();

        for (file, metadata_file_libraries) in self.metadata.libraries.iter() {
            let mut file_path = self.path.clone();
            file_path.pop();
            file_path.push(file);

            let file_path_string = crate::utils::path_to_string_normalized(file_path.as_path());

            let mut file_libraries = BTreeMap::new();
            for name in metadata_file_libraries.keys() {
                let address = address_iterator.next(
                    &web3::types::Address::from_str(DEFAULT_CALLER_ADDRESS).expect("Always valid"),
                    true,
                );
                file_libraries.insert(
                    name.to_owned(),
                    format!("0x{}", crate::utils::address_as_string(&address)),
                );
                library_addresses.insert(format!("{file_path_string}:{name}"), address);
            }
            libraries.insert(file_path_string, file_libraries);
        }

        (libraries.into(), library_addresses)
    }
}

impl Buildable for MatterLabsTest {
    fn build_for_evm(
        &self,
        mode: Mode,
        compiler: Arc<dyn Compiler>,
        summary: Arc<Mutex<Summary>>,
        filters: &Filters,
        debug_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> Option<Test> {
        self.check_filters(filters, &mode)?;

        let mut contracts = self.metadata.contracts.clone();
        self.push_default_contract(&mut contracts, compiler.allows_multi_contract_files());

        let mut address_iterator = AddressIterator::default();

        let sources = self.sources.to_owned();
        let (libraries, library_addresses) = self.get_libraries(&mut address_iterator);

        let test_description = TestDescription {
            group: None,
            mode: Some(mode.clone()),
            selector: self.selector.clone(),
        };

        let evm_input = match compiler
            .compile_for_evm(
                self.selector.path.to_string(),
                sources,
                libraries,
                &mode,
                None,
                vec![],
                debug_config,
            )
            .map_err(|error| anyhow::anyhow!("Failed to compile sources:\n{error}"))
        {
            Ok(output) => output,
            Err(error) => {
                Summary::invalid(summary, test_description, error);
                return None;
            }
        };

        let mut instances = match evm_input.get_instances(&contracts, library_addresses, None) {
            Ok(instances) => instances,
            Err(error) => {
                Summary::invalid(summary, test_description, error);
                return None;
            }
        };

        let mut cases = Vec::with_capacity(self.metadata.cases.len());
        for case in self.metadata.cases.iter() {
            if let Some(filters) = case.modes.as_ref()
                && !mode.check_extended_filters(filters.as_slice())
            {
                continue;
            }

            let case = match case.to_owned().normalize(&contracts, &instances) {
                Ok(case) => case,
                Err(error) => {
                    Summary::invalid(summary, test_description, error);
                    return None;
                }
            };

            match case.set_variables(&mut instances, address_iterator.clone(), &mode) {
                Ok(_) => {}
                Err(error) => {
                    Summary::invalid(summary, test_description, error);
                    return None;
                }
            }

            let case_name = case.name.to_owned();
            let case = match Case::try_from_matter_labs(
                case,
                &mode,
                &instances,
                &evm_input.method_identifiers,
            )
            .map_err(|error| anyhow::anyhow!("Case `{case_name}` is invalid: {error}"))
            {
                Ok(case) => case,
                Err(error) => {
                    Summary::invalid(summary, test_description, error);
                    return None;
                }
            };

            cases.push(case);
        }

        Some(Test::new(
            self.selector.to_string(),
            cases,
            mode,
            self.metadata.group.clone(),
        ))
    }
}
