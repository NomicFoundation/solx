//!
//! The Solidity test adapter library.
//!

#![allow(clippy::assigning_clones)]

pub mod index;
pub mod test;

use alloy_primitives::Address;
use alloy_primitives::U256;

pub use self::index::FSEntity;
pub use self::index::enabled::EnabledTest;
pub use self::test::Test;
pub use self::test::function_call::FunctionCall;
pub use self::test::function_call::event::Event;
pub use self::test::params::Params;
pub use self::test::params::abi_encoder_v1_only::ABIEncoderV1Only;
pub use self::test::params::compile_via_yul::CompileViaYul;
pub use self::test::params::evm_version::EVMVersion;
pub use self::test::params::revert_strings::RevertStrings;

/// The default contract address.
pub const DEFAULT_CONTRACT_ADDRESS: &str = "c06afe3a8444fc0004668591e8306bfb9968e79e";

/// The index of the account used as the default caller.
pub const DEFAULT_ACCOUNT_INDEX: usize = 0;

/// First pre-generated account address.
const ZERO_ADDRESS: &str = "1212121212121212121212121212120000000012";

/// The caller address multiplier.
const ADDRESS_INDEX_MULTIPLIER: usize = 4096; // 16^3

/// The cross-platform new line character.
#[cfg(windows)]
const NEW_LINE: &str = "\r\n";
#[cfg(not(windows))]
const NEW_LINE: &str = "\n";

///
/// Returns address of pre-generated account by index.
///
pub fn account_address(index: usize) -> Address {
    let address = u256_from_hex_str(ZERO_ADDRESS).expect("Default address");
    let address = address + U256::from(index * ADDRESS_INDEX_MULTIPLIER);

    let bytes = address.to_be_bytes::<{ solx_utils::BYTE_LENGTH_FIELD }>();
    Address::from_slice(
        &bytes[solx_utils::BYTE_LENGTH_FIELD - solx_utils::BYTE_LENGTH_ETH_ADDRESS..],
    )
}

///
/// Parses a bare or `0x`-prefixed hexadecimal string into a `U256`.
///
/// Kept over `U256::from_str` because the lexer emits bare hex, which `from_str` reads as decimal.
///
pub(crate) fn u256_from_hex_str(value: &str) -> anyhow::Result<U256> {
    let value = value.strip_prefix("0x").unwrap_or(value);
    U256::from_str_radix(value, 16)
        .map_err(|error| anyhow::anyhow!("Invalid hexadecimal literal `{value}`: {error}"))
}
