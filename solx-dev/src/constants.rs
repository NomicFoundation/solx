//!
//! Shared constants loaded from ci/constants.toml.
//!

use std::sync::OnceLock;

#[derive(Debug)]
pub(crate) struct CiConstants {
    pub(crate) boost_version: String,
    pub(crate) solidity_version: String,
}

static CONSTANTS: OnceLock<CiConstants> = OnceLock::new();

fn load_constants() -> CiConstants {
    let raw = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../ci/constants.toml"));
    let value: toml::Value = raw.parse().expect("ci/constants.toml should be valid TOML");
    let table = value
        .as_table()
        .expect("ci/constants.toml should contain a TOML table");

    let boost_version = table
        .get("boost_version")
        .and_then(|value| value.as_str())
        .expect("boost_version must be a string in ci/constants.toml")
        .to_string();
    let solidity_version = table
        .get("solidity_version")
        .and_then(|value| value.as_str())
        .expect("solidity_version must be a string in ci/constants.toml")
        .to_string();

    CiConstants {
        boost_version,
        solidity_version,
    }
}

fn constants() -> &'static CiConstants {
    CONSTANTS.get_or_init(load_constants)
}

/// Returns the default Boost version from ci/constants.toml.
pub fn boost_version() -> &'static str {
    constants().boost_version.as_str()
}

/// Returns the default Solidity version from ci/constants.toml.
pub fn solidity_version() -> &'static str {
    constants().solidity_version.as_str()
}
