//!
//! `solx` tester utils.
//!

use std::path::Path;
use std::path::PathBuf;

///
/// Overrides the default formatting for `Address`, which replaces the middle with an ellipsis.
///
pub fn address_as_string(value: &web3::types::Address) -> String {
    hex::encode(value.as_bytes())
}

///
/// Overrides the default formatting for `U256`, which replaces the middle with an ellipsis.
///
pub fn u256_as_string(value: &web3::types::U256) -> String {
    let mut bytes = vec![0; solx_utils::BYTE_LENGTH_FIELD];
    value.to_big_endian(&mut bytes);
    hex::encode(bytes)
}

///
/// Converts `U256` into `Address`.
///
pub fn u256_to_address(value: &web3::types::U256) -> web3::types::Address {
    let mut bytes = vec![0; solx_utils::BYTE_LENGTH_FIELD];
    value.to_big_endian(&mut bytes);
    web3::types::Address::from_slice(&bytes[bytes.len() - solx_utils::BYTE_LENGTH_ETH_ADDRESS..])
}

///
/// Converts `Address` into `U256`.
///
pub fn address_to_u256(address: &web3::types::Address) -> web3::types::U256 {
    let mut buffer = [0u8; solx_utils::BYTE_LENGTH_FIELD];
    buffer[solx_utils::BYTE_LENGTH_FIELD - solx_utils::BYTE_LENGTH_ETH_ADDRESS..]
        .copy_from_slice(address.as_bytes());
    web3::types::U256::from_big_endian(&buffer)
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
