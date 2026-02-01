//!
//! The test input.
//!

pub mod balance;
pub mod calldata;
pub mod deploy;
pub mod identifier;
pub mod output;
pub mod runtime;
pub mod storage;
pub mod storage_empty;
pub mod value;

use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use crate::compilers::mode::Mode;
use crate::directories::matter_labs::test::metadata::case::input::Input as MatterLabsTestInput;
use crate::revm::REVM;
use crate::summary::Summary;
use crate::test::InputContext;
use crate::test::instance::Instance;

use self::balance::Balance;
use self::calldata::Calldata;
use self::deploy::Deploy;
use self::output::Output;
use self::runtime::Runtime;
use self::storage::Storage;
use self::storage_empty::StorageEmpty;
use self::value::Value;

///
/// The test input.
///
#[derive(Debug, Clone)]
pub enum Input {
    /// The EVM contract deploy.
    Deploy(Deploy),
    /// The contract call.
    Runtime(Runtime),
    /// The storage empty check.
    StorageEmpty(StorageEmpty),
    /// Check account balance.
    Balance(Balance),
}

impl Input {
    ///
    /// Try convert from Matter Labs compiler test metadata input.
    ///
    pub fn try_from_matter_labs(
        input: MatterLabsTestInput,
        mode: &Mode,
        instances: &BTreeMap<String, Instance>,
        method_identifiers: &Option<BTreeMap<String, BTreeMap<String, u32>>>,
    ) -> anyhow::Result<Self> {
        let caller = match Value::try_from_matter_labs(input.caller.as_str(), instances)
            .map_err(|error| anyhow::anyhow!("Invalid caller `{}`: {error}", input.caller))?
        {
            Value::Known(value) => crate::utils::u256_to_address(&value),
            Value::Any => anyhow::bail!("Caller can not be `*`"),
        };

        let value = match input.value {
            Some(value) => Some(if let Some(value) = value.strip_suffix(" ETH") {
                u128::from_str(value)
                    .map_err(|error| anyhow::anyhow!("Invalid value literal `{value}`: {error}"))?
                    .checked_mul(10u128.pow(18))
                    .ok_or_else(|| {
                        anyhow::anyhow!("Invalid value literal `{value}`: u128 overflow")
                    })?
            } else if let Some(value) = value.strip_suffix(" wei") {
                u128::from_str(value)
                    .map_err(|error| anyhow::anyhow!("Invalid value literal `{value}`: {error}"))?
            } else {
                anyhow::bail!("Invalid value `{value}`");
            }),
            None => None,
        };

        let mut calldata = Calldata::try_from_matter_labs(input.calldata, instances)
            .map_err(|error| anyhow::anyhow!("Invalid calldata: {error}"))?;

        let expected = match input.expected {
            Some(expected) => Output::try_from_matter_labs_expected(expected, mode, instances)
                .map_err(|error| anyhow::anyhow!("Invalid expected metadata: {error}"))?,
            None => Output::default(),
        };

        let storage = Storage::try_from_matter_labs(input.storage, instances)
            .map_err(|error| anyhow::anyhow!("Invalid storage: {error}"))?;

        let instance = instances
            .get(&input.instance)
            .ok_or_else(|| anyhow::anyhow!("Instance `{}` not found", input.instance))?;

        let input = match input.method.as_str() {
            "#deployer" => Input::Deploy(Deploy::new(
                instance.path.to_owned(),
                instance.deploy_code.to_owned(),
                instance.runtime_code_size,
                calldata,
                caller,
                value,
                expected,
            )),
            "#fallback" => {
                let address = instance.address().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Instance `{}` was not successfully deployed",
                        input.instance
                    )
                })?;

                Input::Runtime(Runtime::new(
                    "#fallback".to_string(),
                    *address,
                    calldata,
                    caller,
                    value,
                    storage,
                    expected,
                ))
            }
            entry => {
                let address = instance.address().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Instance `{}` was not successfully deployed",
                        input.instance
                    )
                })?;

                let path = instance.path();
                let selector = match method_identifiers {
                    Some(method_identifiers) => method_identifiers
                        .get(path)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Contract `{path}` not found in the method identifiers")
                        })?
                        .iter()
                        .find_map(|(name, selector)| {
                            if name.starts_with(entry) {
                                Some(*selector)
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "In contract `{path}`, selector of the method `{entry}` not found"
                            )
                        })?,
                    None => u32::from_str_radix(entry, solx_utils::BASE_HEXADECIMAL).map_err(
                        |error| {
                            anyhow::anyhow!("Invalid entry value for contract `{path}`: {error}")
                        },
                    )?,
                };

                calldata.push_selector(selector);

                Input::Runtime(Runtime::new(
                    entry.to_string(),
                    *address,
                    calldata,
                    caller,
                    value,
                    storage,
                    expected,
                ))
            }
        };

        Ok(input)
    }

    ///
    /// Try convert from Ethereum compiler test metadata input.
    ///
    pub fn try_from_ethereum(
        input: &solx_solc_test_adapter::FunctionCall,
        instances: &BTreeMap<String, Instance>,
        last_source: &str,
        caller: &web3::types::Address,
    ) -> anyhow::Result<Option<Self>> {
        let main_contract_instance = instances
            .values()
            .find(|instance| instance.is_main())
            .ok_or_else(|| anyhow::anyhow!("Could not identify the Ethereum test main contract"))?
            .to_owned();
        let main_contract_address = main_contract_instance.address().expect("Always exists");

        let input = match input {
            solx_solc_test_adapter::FunctionCall::Constructor {
                calldata,
                value,
                events,
                ..
            } => {
                let value = match value {
                    Some(value) => Some((*value).try_into().map_err(|error| {
                        anyhow::anyhow!("Invalid value literal `{value:X}`: {error}")
                    })?),
                    None => None,
                };

                let expected = Output::from_ethereum_expected(
                    &[web3::types::U256::from_big_endian(
                        main_contract_address.as_bytes(),
                    )],
                    false,
                    events,
                    main_contract_address,
                );

                Some(Input::Deploy(Deploy::new(
                    main_contract_instance.path.to_owned(),
                    main_contract_instance.deploy_code.to_owned(),
                    main_contract_instance.runtime_code_size,
                    calldata.clone().into(),
                    *caller,
                    value,
                    expected,
                )))
            }
            solx_solc_test_adapter::FunctionCall::Library { name, source } => {
                let source = crate::utils::str_to_string_normalized(
                    source.as_deref().unwrap_or(last_source),
                );
                let library = format!("{source}:{name}");
                let instance = instances
                    .get(library.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Library `{library}` not found"))?;

                let expected = Output::from_ethereum_expected(
                    &[web3::types::U256::from_big_endian(
                        instance
                            .address()
                            .expect("Must be set by this point")
                            .as_bytes(),
                    )],
                    false,
                    &[],
                    main_contract_address,
                );

                Some(Input::Deploy(Deploy::new(
                    instance.path.to_owned(),
                    instance.deploy_code.to_owned(),
                    instance.runtime_code_size,
                    Calldata::default(),
                    *caller,
                    None,
                    expected,
                )))
            }
            solx_solc_test_adapter::FunctionCall::Balance {
                input, expected, ..
            } => {
                let address = input.unwrap_or(*main_contract_address);
                Some(Input::Balance(Balance::new(address, *expected)))
            }
            solx_solc_test_adapter::FunctionCall::StorageEmpty { expected } => {
                Some(Input::StorageEmpty(StorageEmpty::new(*expected)))
            }
            solx_solc_test_adapter::FunctionCall::Call {
                method,
                calldata,
                value,
                expected,
                failure,
                events,
                ..
            } => {
                let value = match value {
                    Some(value) => Some((*value).try_into().map_err(|error| {
                        anyhow::anyhow!("Invalid value literal `{value:X}`: {error}")
                    })?),
                    None => None,
                };

                let expected = Output::from_ethereum_expected(
                    expected,
                    *failure,
                    events,
                    main_contract_address,
                );

                Some(Input::Runtime(Runtime::new(
                    method.clone(),
                    *main_contract_address,
                    calldata.clone().into(),
                    *caller,
                    value,
                    Storage::default(),
                    expected,
                )))
            }
            _ => None,
        };

        Ok(input)
    }

    ///
    /// Runs the input on REVM.
    ///
    pub fn run_revm(self, summary: Arc<Mutex<Summary>>, vm: &mut REVM, context: InputContext<'_>) {
        match self {
            Self::Deploy(deploy) => deploy.run_revm(summary, vm, context),
            Self::Runtime(runtime) => runtime.run_revm(summary, vm, context),
            Self::StorageEmpty(storage_empty) => storage_empty.run_revm(summary, vm, context),
            Self::Balance(balance_check) => balance_check.run_revm(summary, vm, context),
        }
    }
}
