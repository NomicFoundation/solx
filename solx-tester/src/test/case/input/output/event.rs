//!
//! The compiler test outcome event.
//!

use std::collections::BTreeMap;
use std::str::FromStr;

use crate::directories::matter_labs::test::metadata::case::input::expected::variant::extended::event::Event as MatterLabsTestExpectedEvent;
use crate::test::instance::Instance;
use crate::test::case::input::value::Value;

///
/// The compiler test outcome event.
///
#[derive(Debug, Clone, serde::Serialize)]
pub struct Event {
    /// The event address.
    address: Option<web3::types::Address>,
    /// The event topics.
    topics: Vec<Value>,
    /// The event values.
    values: Vec<Value>,
}

impl Event {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        address: Option<web3::types::Address>,
        topics: Vec<Value>,
        values: Vec<Value>,
    ) -> Self {
        Self {
            address,
            topics,
            values,
        }
    }

    ///
    /// Try convert from Matter Labs compiler test metadata expected event.
    ///
    pub fn try_from_matter_labs(
        event: MatterLabsTestExpectedEvent,
        instances: &BTreeMap<String, Instance>,
    ) -> anyhow::Result<Self> {
        let topics = Value::try_from_vec_matter_labs(event.topics, instances)
            .map_err(|error| anyhow::anyhow!("Invalid topics: {error}"))?;
        let values = Value::try_from_vec_matter_labs(event.values, instances)
            .map_err(|error| anyhow::anyhow!("Invalid values: {error}"))?;

        let address = match event.address {
            Some(address) => Some(
                if let Some(instance) = address.strip_suffix(".address") {
                    instances
                        .get(instance)
                        .ok_or_else(|| anyhow::anyhow!("Instance `{instance}` not found"))?
                        .address()
                        .copied()
                        .ok_or_else(|| {
                            anyhow::anyhow!("Instance `{instance}` was not successfully deployed")
                        })
                } else {
                    web3::types::Address::from_str(address.as_str())
                        .map_err(|error| anyhow::anyhow!("Invalid address literal: {error}"))
                }
                .map_err(|error| anyhow::anyhow!("Invalid event address `{address}`: {error}"))?,
            ),
            None => None,
        };

        Ok(Self {
            address,
            topics,
            values,
        })
    }

    ///
    /// Convert from Ethereum compiler test metadata expected event.
    ///
    pub fn from_ethereum(
        event: &solx_solc_test_adapter::Event,
        contract_address: &web3::types::Address,
    ) -> Self {
        let topics = event
            .topics
            .iter()
            .map(|topic| {
                let mut topic_str = crate::utils::u256_as_string(topic);
                topic_str = topic_str.replace(
                    solx_solc_test_adapter::DEFAULT_CONTRACT_ADDRESS,
                    &crate::utils::address_as_string(contract_address),
                );
                Value::Known(
                    web3::types::U256::from_str(&topic_str)
                        .expect("Solidity adapter default contract address constant is invalid"),
                )
            })
            .collect();

        let values = event
            .expected
            .iter()
            .map(|value| {
                let mut value_str = crate::utils::u256_as_string(value);
                value_str = value_str.replace(
                    solx_solc_test_adapter::DEFAULT_CONTRACT_ADDRESS,
                    &crate::utils::address_as_string(contract_address),
                );
                Value::Known(
                    web3::types::U256::from_str(&value_str)
                        .expect("Solidity adapter default contract address constant is invalid"),
                )
            })
            .collect();

        Self {
            // The address is ignored, as Ethereum tests expect other addresses
            address: None,
            topics,
            values,
        }
    }
}

impl PartialEq<Self> for Event {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(address1), Some(address2)) = (self.address, other.address)
            && address1 != address2
        {
            return false;
        };

        if self.topics.len() != other.topics.len() {
            return false;
        }
        if self.values.len() != other.values.len() {
            return false;
        }

        for index in 0..self.topics.len() {
            if let (Value::Known(value1), Value::Known(value2)) =
                (&self.topics[index], &other.topics[index])
                && value1 != value2
            {
                return false;
            }
        }

        for index in 0..self.values.len() {
            if let (Value::Known(value1), Value::Known(value2)) =
                (&self.values[index], &other.values[index])
                && value1 != value2
            {
                return false;
            }
        }

        true
    }
}
