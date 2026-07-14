//!
//! `solx` tester utils.
//!

use std::path::Path;
use std::path::PathBuf;

use revm::primitives::Address;
use revm::primitives::U256;

///
/// Overrides the default formatting for `Address`, which replaces the middle with an ellipsis.
///
pub fn address_as_string(value: &Address) -> String {
    hex::encode(value.as_slice())
}

///
/// Overrides the default formatting for `U256`, which replaces the middle with an ellipsis.
///
pub fn u256_as_string(value: &U256) -> String {
    hex::encode(value.to_be_bytes::<{ solx_utils::BYTE_LENGTH_FIELD }>())
}

///
/// Converts `U256` into `Address`.
///
pub fn u256_to_address(value: &U256) -> Address {
    let bytes = value.to_be_bytes::<{ solx_utils::BYTE_LENGTH_FIELD }>();
    Address::from_slice(&bytes[bytes.len() - solx_utils::BYTE_LENGTH_ETH_ADDRESS..])
}

///
/// Converts `Address` into `U256`.
///
pub fn address_to_u256(address: &Address) -> U256 {
    let mut buffer = [0u8; solx_utils::BYTE_LENGTH_FIELD];
    buffer[solx_utils::BYTE_LENGTH_FIELD - solx_utils::BYTE_LENGTH_ETH_ADDRESS..]
        .copy_from_slice(address.as_slice());
    U256::from_be_bytes(buffer)
}

///
/// Parses a hexadecimal string (with optional `0x` prefix, no checksum validation) into a `U256`.
///
/// Preserves the lenient hex parsing that `web3`'s `U256::from_str` previously provided.
///
pub fn u256_from_hex_str(value: &str) -> anyhow::Result<U256> {
    let value = value.strip_prefix("0x").unwrap_or(value);
    U256::from_str_radix(value, 16)
        .map_err(|error| anyhow::anyhow!("Invalid hexadecimal literal `{value}`: {error}"))
}

///
/// Parses a hexadecimal address (with optional `0x` prefix, no checksum validation) into an `Address`.
///
/// Preserves the lenient hex parsing that `web3`'s `Address::from_str` previously provided.
///
pub fn address_from_hex_str(value: &str) -> anyhow::Result<Address> {
    let value = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(value)
        .map_err(|error| anyhow::anyhow!("Invalid address literal `{value}`: {error}"))?;
    if bytes.len() != solx_utils::BYTE_LENGTH_ETH_ADDRESS {
        anyhow::bail!(
            "Invalid address literal `{value}`: expected {} bytes, got {}",
            solx_utils::BYTE_LENGTH_ETH_ADDRESS,
            bytes.len()
        );
    }
    Ok(Address::from_slice(&bytes))
}

///
/// Normalizes `path` by replacing possible backslashes with ordinar slashes, and returns a string.
///
pub fn path_to_string_normalized(path: &Path) -> String {
    path.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR_STR, "/")
}

///
/// Normalizes `path` by replacing possible backslashes with ordinar slashes, and returns a `PathBuf`.
///
pub fn str_to_path_normalized(path: &str) -> PathBuf {
    PathBuf::from(self::str_to_string_normalized(path))
}

///
/// Normalizes stringified `path` by replacing possible backslashes with ordinar slashes, and returns a string.
///
pub fn str_to_string_normalized(path: &str) -> String {
    path.replace(std::path::MAIN_SEPARATOR_STR, "/")
}
