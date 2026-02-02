//!
//! The compiler test outcome data.
//!

pub mod event;

use std::collections::BTreeMap;
use std::str::FromStr;

use crate::compilers::mode::Mode;
use crate::directories::matter_labs::test::metadata::case::input::expected::Expected as MatterLabsTestExpected;
use crate::directories::matter_labs::test::metadata::case::input::expected::variant::Variant as MatterLabsTestExpectedVariant;
use crate::revm::revm_type_conversions::revm_bytes_to_vec_value;
use crate::revm::revm_type_conversions::revm_topics_to_vec_value;
use crate::test::case::input::value::Value;
use crate::test::instance::Instance;

use self::event::Event;

///
/// The compiler test outcome data.
///
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Output {
    /// The return data values.
    pub return_data: Vec<Value>,
    /// Whether an exception is thrown,
    pub exception: bool,
    /// The emitted events.
    pub events: Vec<Event>,
}

impl Output {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(return_data: Vec<Value>, exception: bool, events: Vec<Event>) -> Self {
        Self {
            return_data,
            exception,
            events,
        }
    }

    ///
    /// Try convert from Matter Labs compiler test metadata expected.
    ///
    pub fn try_from_matter_labs_expected(
        expected: MatterLabsTestExpected,
        mode: &Mode,
        instances: &BTreeMap<String, Instance>,
    ) -> anyhow::Result<Self> {
        let variants = match expected {
            MatterLabsTestExpected::Single(variant) => vec![variant],
            MatterLabsTestExpected::Multiple(variants) => variants.into_iter().collect(),
        };
        let variant = variants
            .into_iter()
            .find(|variant| {
                let version = match variant {
                    MatterLabsTestExpectedVariant::Simple(_) => None,
                    MatterLabsTestExpectedVariant::Extended(inner) => {
                        inner.compiler_version.as_ref()
                    }
                };
                match version {
                    Some(version) => mode.check_version(version),
                    None => true,
                }
            })
            .ok_or_else(|| anyhow::anyhow!("Version not covered"))?;

        let (return_data, exception, events) = match variant {
            MatterLabsTestExpectedVariant::Simple(return_data) => (return_data, false, Vec::new()),
            MatterLabsTestExpectedVariant::Extended(expected) => {
                let return_data = expected.return_data;
                let exception = expected.exception;
                let events = expected
                    .events
                    .into_iter()
                    .enumerate()
                    .map(|(index, event)| {
                        Event::try_from_matter_labs(event, instances)
                            .map_err(|error| anyhow::anyhow!("Event #{index} is invalid: {error}"))
                    })
                    .collect::<anyhow::Result<Vec<Event>>>()
                    .map_err(|error| anyhow::anyhow!("Invalid events: {error}"))?;
                (return_data, exception, events)
            }
        };
        let return_data = Value::try_from_vec_matter_labs(return_data, instances)
            .map_err(|error| anyhow::anyhow!("Invalid return data: {error}"))?;

        Ok(Self {
            return_data,
            exception,
            events,
        })
    }

    ///
    /// Convert from Ethereum compiler test metadata expected.
    ///
    pub fn from_ethereum_expected(
        expected: &[web3::types::U256],
        exception: bool,
        events: &[solx_solc_test_adapter::Event],
        contract_address: &web3::types::Address,
    ) -> Self {
        let return_data = expected
            .iter()
            .map(|value| {
                Value::Known(
                    web3::types::U256::from_str(&crate::utils::u256_as_string(value))
                        .expect("Solidity adapter default contract address constant is invalid"),
                )
            })
            .collect();

        let events = events
            .iter()
            .map(|event| Event::from_ethereum(event, contract_address))
            .collect();

        Self {
            return_data,
            exception,
            events,
        }
    }
}

impl From<web3::types::U256> for Output {
    fn from(value: web3::types::U256) -> Self {
        Self {
            return_data: vec![Value::Known(value)],
            exception: false,
            events: vec![],
        }
    }
}

impl From<bool> for Output {
    fn from(value: bool) -> Self {
        let value = if value {
            web3::types::U256::one()
        } else {
            web3::types::U256::zero()
        };
        value.into()
    }
}

impl From<(revm::context::result::Output, Vec<revm::primitives::Log>)> for Output {
    fn from((output, logs): (revm::context::result::Output, Vec<revm::primitives::Log>)) -> Self {
        let bytes = match output {
            revm::context::result::Output::Call(bytes) => bytes,
            revm::context::result::Output::Create(_, address) => {
                let addr_slice = address.unwrap();
                revm::primitives::Bytes::from(addr_slice.into_word())
            }
        };
        let return_data_value = revm_bytes_to_vec_value(bytes);

        let events = logs
            .into_iter()
            .map(|log| {
                let topics = revm_topics_to_vec_value(log.data.topics());
                let data_value = revm_bytes_to_vec_value(log.data.data);
                Event::new(
                    Some(web3::types::Address::from_slice(log.address.as_slice())),
                    topics,
                    data_value,
                )
            })
            .collect();
        Self::new(return_data_value, false, events)
    }
}

impl PartialEq<Self> for Output {
    fn eq(&self, other: &Self) -> bool {
        if self.exception != other.exception {
            return false;
        }
        if self.events.len() != other.events.len() {
            return false;
        }
        if self.return_data.len() != other.return_data.len() {
            return false;
        }

        for index in 0..self.return_data.len() {
            if let (Value::Known(value_1), Value::Known(value_2)) =
                (&self.return_data[index], &other.return_data[index])
                && value_1 != value_2
            {
                return false;
            }
        }

        for index in 0..self.events.len() {
            if self.events[index] != other.events[index] {
                return false;
            }
        }

        true
    }
}
