//!
//! The compiler test value.
//!

use std::collections::BTreeMap;
use std::str::FromStr;

use serde::Serialize;
use serde::Serializer;

use crate::revm::REVM;
use crate::test::instance::Instance;

///
/// The compiler test value.
///
#[derive(Debug, Clone)]
pub enum Value {
    /// Any value (used for expected data).
    Any,
    /// The known value.
    Known(web3::types::U256),
}

impl Value {
    ///
    /// Try convert from Matter Labs compiler test metadata value.
    ///
    pub fn try_from_matter_labs(
        value: &str,
        instances: &BTreeMap<String, Instance>,
    ) -> anyhow::Result<Self> {
        if value == "*" {
            return Ok(Self::Any);
        }

        let value = if let Some(instance) = value.strip_suffix(".address") {
            web3::types::U256::from_big_endian(
                instances
                    .get(instance)
                    .ok_or_else(|| anyhow::anyhow!("Instance `{instance}` not found"))?
                    .address()
                    .ok_or_else(|| {
                        anyhow::anyhow!("Instance `{instance}` was not successfully deployed")
                    })?
                    .as_bytes(),
            )
        } else if let Some(value) = value.strip_prefix('-') {
            let value = web3::types::U256::from_dec_str(value)
                .map_err(|error| anyhow::anyhow!("Invalid decimal literal after `-`: {error}"))?;
            if value > web3::types::U256::one() << 255u8 {
                anyhow::bail!("Decimal literal after `-` is too big");
            }
            let value = value
                .checked_sub(web3::types::U256::one())
                .ok_or_else(|| anyhow::anyhow!("`-0` is invalid literal"))?;
            web3::types::U256::max_value()
                .checked_sub(value)
                .expect("Always valid")
        } else if let Some(value) = value.strip_prefix("0x") {
            web3::types::U256::from_str(value)
                .map_err(|error| anyhow::anyhow!("Invalid hexadecimal literal: {error}"))?
        } else if value == "$CHAIN_ID" {
            web3::types::U256::from(REVM::CHAIND_ID)
        } else if value == "$GAS_LIMIT" {
            web3::types::U256::from(REVM::BLOCK_GAS_LIMIT)
        } else if value == "$COINBASE" {
            web3::types::U256::from_str_radix(REVM::COIN_BASE, solx_utils::BASE_HEXADECIMAL)
                .expect("Always valid")
        } else if value == "$PREVRANDAO" {
            web3::types::U256::from(REVM::BLOCK_PREVRANDAO)
        } else if value.starts_with("$BLOCK_HASH") {
            let offset: u64 = value
                .split(':')
                .next_back()
                .and_then(|value| value.parse().ok())
                .unwrap_or_default();
            let mut hash = web3::types::U256::from_str(REVM::BLOCK_HASH).expect("Always valid");
            hash += web3::types::U256::from(offset);
            hash
        } else if value == "$BLOCK_NUMBER" {
            web3::types::U256::from(REVM::BLOCK_NUMBER)
        } else if value == "$BLOCK_TIMESTAMP" {
            web3::types::U256::from(REVM::BLOCK_TIMESTAMP)
        } else if value == "$TX_ORIGIN" {
            crate::utils::address_to_u256(
                &web3::types::Address::from_str(REVM::TX_ORIGIN).expect("Alwyays valid"),
            )
        } else if value == "$BASE_FEE" {
            web3::types::U256::from(REVM::BASE_FEE)
        } else if value == "$GAS_PRICE" {
            web3::types::U256::from(REVM::GAS_PRICE)
        } else {
            web3::types::U256::from_dec_str(value)
                .map_err(|error| anyhow::anyhow!("Invalid decimal literal: {error}"))?
        };

        Ok(Self::Known(value))
    }

    ///
    /// Try convert into vec of self from vec of Matter Labs compiler test metadata values.
    ///
    pub fn try_from_vec_matter_labs(
        values: Vec<String>,
        instances: &BTreeMap<String, Instance>,
    ) -> anyhow::Result<Vec<Self>> {
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                Self::try_from_matter_labs(value.as_str(), instances)
                    .map_err(|error| anyhow::anyhow!("Value {index} is invalid: {error}"))
            })
            .collect::<anyhow::Result<Vec<Self>>>()
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value_str = match self {
            Value::Known(value) => format!("0x{}", crate::utils::u256_as_string(value)),
            Value::Any => "*".to_string(),
        };
        serializer.serialize_str(&value_str)
    }
}
