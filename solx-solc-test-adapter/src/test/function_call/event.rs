//!
//! The event.
//!

use std::str::FromStr;

use crate::test::function_call::parser::Event as SyntaxEvent;
use crate::test::function_call::parser::EventVariant;

///
/// The event.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// The event address.
    pub address: Option<web3::types::Address>,
    /// The event topics.
    pub topics: Vec<web3::types::U256>,
    /// The expected values.
    pub expected: Vec<web3::types::U256>,
}

impl TryFrom<SyntaxEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(event: SyntaxEvent) -> Result<Self, Self::Error> {
        let address = event.address.as_ref().map(|address| {
            web3::types::Address::from_str(address).expect(super::VALIDATED_BY_THE_PARSER)
        });
        let mut expected = Vec::new();
        let mut topics = Vec::new();
        if let Some(literals) = event.expected {
            for literal in literals {
                if literal.indexed {
                    topics.extend(literal.inner.as_bytes_be()?);
                } else {
                    expected.extend(literal.inner.as_bytes_be()?);
                }
            }
        }
        let mut topics = super::FunctionCall::bytes_as_u256(topics.as_slice());
        if let EventVariant::Signature { identifier, types } = event.variant {
            topics.insert(
                0,
                web3::types::U256::from_big_endian(
                    web3::signing::keccak256(
                        super::FunctionCall::signature(Some(&identifier), Some(types.as_slice()))
                            .as_bytes(),
                    )
                    .as_slice(),
                ),
            )
        }
        let expected = super::FunctionCall::bytes_as_u256(expected.as_slice());
        Ok(Self {
            address,
            topics,
            expected,
        })
    }
}
