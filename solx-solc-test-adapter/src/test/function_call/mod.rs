//!
//! The function call.
//!

pub mod event;
pub mod parser;

use crate::test::function_call::parser::Call;
use crate::test::function_call::parser::CallVariant;
use crate::test::function_call::parser::Identifier;
use crate::test::function_call::parser::Type;
use crate::test::function_call::parser::Unit;
use crate::test::function_call::parser::syntax::tree::literal::Literal;
use crate::test::function_call::parser::syntax::tree::value::Value;

use self::event::Event;

///
/// The function call.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionCall {
    /// The library.
    Library {
        /// The library name.
        name: String,
        /// The source file name.
        source: Option<String>,
    },
    /// The custom function call.
    Call {
        /// The function signature.
        method: String,
        /// The calldata.
        calldata: Vec<u8>,
        /// The value in wei.
        value: Option<web3::types::U256>,
        /// The expected output.
        expected: Vec<web3::types::U256>,
        /// The flag if failure expected.
        failure: bool,
        /// The expected events.
        events: Vec<Event>,
    },
    /// The constructor call.
    Constructor {
        /// The calldata.
        calldata: Vec<u8>,
        /// The value in wei.
        value: Option<web3::types::U256>,
        /// The expected events.
        events: Vec<Event>,
    },
    /// The `isoltest_builtin_test` standard function.
    IsoltestBuiltinTest {
        /// The expected return value.
        expected: web3::types::U256,
    },
    /// The `isoltest_side_effects_test` standard function.
    IsoltestSideEffectsTest {
        /// The input.
        input: Vec<u8>,
        /// The expected output.
        expected: Vec<web3::types::U256>,
    },
    /// The `balance` standard function.
    Balance {
        /// The input.
        input: Option<web3::types::Address>,
        /// The expected output.
        expected: web3::types::U256,
        /// The expected events.
        events: Vec<Event>,
    },
    /// The `storageEmpty` standard function.
    StorageEmpty {
        /// The expected output.
        expected: bool,
    },
    /// The `account` standard function.
    Account {
        /// The input.
        input: usize,
        /// The expected output.
        expected: web3::types::Address,
    },
}

impl TryFrom<parser::Call> for FunctionCall {
    type Error = anyhow::Error;

    fn try_from(value: Call) -> anyhow::Result<Self> {
        match value.variant {
            CallVariant::Library { identifier, source } => Ok(Self::Library {
                name: identifier.name,
                source,
            }),
            CallVariant::Call {
                identifier,
                types,
                value,
                input,
                expected,
                failure,
                events,
            } => {
                let signature = Self::signature(identifier.as_ref(), types.as_deref());
                let value = Self::parse_value(value)?;
                let input = Self::parse_input(input)?;
                let expected = Self::parse_expected(expected)?;
                let events = Self::parse_events(events)?;

                match signature.as_str() {
                    "constructor()" => {
                        if !expected.is_empty() {
                            anyhow::bail!("Constructor should not expect values");
                        }
                        Ok(Self::Constructor {
                            calldata: input,
                            value,
                            events,
                        })
                    }
                    "isoltest_builtin_test" => {
                        if expected.len() != 1 {
                            anyhow::bail!("isoltest_builtin_test should expect one element");
                        }
                        if !input.is_empty() {
                            anyhow::bail!("isoltest_builtin_test don't expect params");
                        }
                        if !events.is_empty() {
                            anyhow::bail!("standard functions don't emit events");
                        }
                        Ok(Self::IsoltestBuiltinTest {
                            expected: expected.into_iter().next().expect("length checked above"),
                        })
                    }
                    "isoltest_side_effects_test" => {
                        if !events.is_empty() {
                            anyhow::bail!("standard functions don't emit events");
                        }
                        Ok(Self::IsoltestSideEffectsTest { input, expected })
                    }
                    "balance" => {
                        if input.len() > solx_utils::BYTE_LENGTH_FIELD {
                            anyhow::bail!("balance function expect one or zero element");
                        }
                        if expected.len() != 1 {
                            anyhow::bail!("balance function returns 1 element");
                        }
                        Ok(Self::Balance {
                            input: if input.is_empty() {
                                None
                            } else {
                                if !input
                                    .iter()
                                    .take(
                                        solx_utils::BYTE_LENGTH_FIELD
                                            - solx_utils::BYTE_LENGTH_ETH_ADDRESS,
                                    )
                                    .all(|byte| byte.eq(&0))
                                {
                                    anyhow::bail!(
                                        "expected cleaned up address as input for balance function"
                                    );
                                }
                                Some(web3::types::Address::from_slice(
                                    &input[solx_utils::BYTE_LENGTH_FIELD
                                        - solx_utils::BYTE_LENGTH_ETH_ADDRESS..],
                                ))
                            },
                            expected: expected.into_iter().next().expect("length checked above"),
                            events,
                        })
                    }
                    "storageEmpty" => {
                        if !input.is_empty() {
                            anyhow::bail!("storageEmpty function don't expect input");
                        }
                        if expected.len() != 1 {
                            anyhow::bail!("storageEmpty function returns one element");
                        }
                        if !events.is_empty() {
                            anyhow::bail!("standard functions don't emit events");
                        }
                        Ok(Self::StorageEmpty {
                            expected: !expected
                                .into_iter()
                                .next()
                                .expect("length checked above")
                                .is_zero(),
                        })
                    }
                    "account" => {
                        if input.len() != solx_utils::BYTE_LENGTH_FIELD {
                            anyhow::bail!("account function expect one element");
                        }
                        if expected.len() != 1 {
                            anyhow::bail!("account function returns 1 element");
                        }
                        if !events.is_empty() {
                            anyhow::bail!("standard functions don't emit events");
                        }
                        let input = web3::types::U256::from_big_endian(input.as_slice());
                        let expected = expected.into_iter().next().expect("length checked above");
                        let mut expected_bytes = [0u8; solx_utils::BYTE_LENGTH_FIELD];
                        expected.to_big_endian(&mut expected_bytes);
                        Ok(Self::Account {
                            input: input.as_usize(),
                            expected: web3::types::Address::from_slice(
                                &expected_bytes
                                    [expected_bytes.len() - solx_utils::BYTE_LENGTH_ETH_ADDRESS..],
                            ),
                        })
                    }
                    signature_str => {
                        let calldata = if signature == "()" {
                            input
                        } else {
                            let mut bytes =
                                web3::signing::keccak256(signature_str.as_bytes())[0..4].to_vec();
                            bytes.extend(input);
                            bytes
                        };
                        Ok(Self::Call {
                            method: identifier
                                .map(|identifier| identifier.name)
                                .unwrap_or_default(),
                            calldata,
                            value,
                            expected,
                            failure,
                            events,
                        })
                    }
                }
            }
        }
    }
}

impl FunctionCall {
    ///
    /// Parses function calls.
    ///
    pub fn parse_calls(value: &str) -> anyhow::Result<Vec<Self>> {
        self::parser::Parser::default()
            .parse(value)
            .map_err(|error| anyhow::anyhow!("Failed to parse function calls: {error:?}"))?
            .into_iter()
            .map(|call| call.try_into())
            .collect::<anyhow::Result<Vec<FunctionCall>>>()
    }
    ///
    /// Parses value option into wei amount.
    ///
    fn parse_value(value: Option<Value>) -> anyhow::Result<Option<web3::types::U256>> {
        match value {
            Some(value) => {
                let mut amount = web3::types::U256::from_dec_str(value.amount.as_str())
                    .expect(VALIDATED_BY_THE_PARSER);
                if value.unit == Unit::Ether {
                    amount = amount
                        .checked_mul(web3::types::U256::from(u64::pow(10, 18)))
                        .ok_or_else(|| anyhow::anyhow!("Overflow: amount too much"))?;
                }
                Ok(Some(amount))
            }
            None => Ok(None),
        }
    }

    ///
    /// Parses literals into bytes.
    ///
    fn parse_input(input: Option<Vec<Literal>>) -> anyhow::Result<Vec<u8>> {
        Ok(match input {
            Some(input) => input
                .into_iter()
                .map(|literal| literal.as_bytes_be())
                .collect::<anyhow::Result<Vec<Vec<u8>>>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<u8>>(),
            None => Vec::new(),
        })
    }

    ///
    /// Parses expected literals into U256 values.
    ///
    fn parse_expected(expected: Option<Vec<Literal>>) -> anyhow::Result<Vec<web3::types::U256>> {
        let expected = Self::parse_input(expected)?;
        Ok(Self::bytes_as_u256(expected.as_slice()))
    }

    ///
    /// Parses expected events.
    ///
    fn parse_events(events: Vec<parser::Event>) -> anyhow::Result<Vec<Event>> {
        events
            .into_iter()
            .map(|event| event.try_into())
            .collect::<anyhow::Result<Vec<Event>>>()
    }

    ///
    /// Returns signature from identifier and types.
    ///
    fn signature(identifier: Option<&Identifier>, types: Option<&[Type]>) -> String {
        let mut signature = identifier
            .map(|identifier| identifier.name.clone())
            .unwrap_or_default();
        if let Some(types) = types {
            signature.push_str(
                format!(
                    "({})",
                    types
                        .iter()
                        .map(|r#type| r#type.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
                .as_str(),
            );
        }
        signature
    }

    ///
    /// Converts bytes to vector of U256.
    ///
    fn bytes_as_u256(bytes: &[u8]) -> Vec<web3::types::U256> {
        let mut result = Vec::new();
        for value in bytes.chunks(solx_utils::BYTE_LENGTH_FIELD) {
            let mut value = value.to_owned();
            while value.len() < solx_utils::BYTE_LENGTH_FIELD {
                value.push(0);
            }
            result.push(web3::types::U256::from_big_endian(value.as_slice()));
        }
        result
    }
}

/// The unreachable branch panic, which is prevented by the parser.
static VALIDATED_BY_THE_PARSER: &str = "Unreachable as long as the parser works correctly";
