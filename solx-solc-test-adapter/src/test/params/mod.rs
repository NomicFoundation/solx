//!
//! The test file params.
//!

pub mod abi_encoder_v1_only;
pub mod allow_non_existing_functions;
pub mod compile_via_yul;
pub mod evm_version;
pub mod revert_strings;

use regex::Regex;

use self::abi_encoder_v1_only::ABIEncoderV1Only;
use self::allow_non_existing_functions::AllowNonExistingFunctions;
use self::compile_via_yul::CompileViaYul;
use self::evm_version::EVMVersion;
use self::revert_strings::RevertStrings;

///
/// The test file params.
///
#[derive(Debug, PartialEq, Eq)]
pub struct Params {
    /// The compileViaYul param value.
    pub compile_via_yul: CompileViaYul,
    /// The ABIEncoderV1Only param value.
    pub abi_encoder_v1_only: ABIEncoderV1Only,
    /// EVM versions param value.
    pub evm_version: EVMVersion,
    /// revertStrings param value.
    pub revert_strings: RevertStrings,
    /// allowNonExistingFunctions param value.
    pub allow_non_existing_functions: AllowNonExistingFunctions,
    /// bytecodeFormat param value.
    pub bytecode_format: String,
}

impl TryFrom<&str> for Params {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut compile_via_yul = CompileViaYul::Default;
        let mut abi_encoder_v1_only = ABIEncoderV1Only::Default;
        let mut evm_version = EVMVersion::Default;
        let mut revert_strings = RevertStrings::Default;
        let mut allow_non_existing_functions = AllowNonExistingFunctions::Default;
        let mut bytecode_format = String::new();

        let regex = Regex::new("^(.*): (.*)$").expect("regex is compile-time constant");
        for (index, line) in value.lines().enumerate() {
            let captures = regex.captures(line).ok_or_else(|| {
                anyhow::anyhow!(
                    "Expected option description on line: {}, found: {}",
                    index + 1,
                    line
                )
            })?;

            let param = captures
                .get(1)
                .expect("capture group 1 always present in matched pattern")
                .as_str();
            let value = captures
                .get(2)
                .expect("capture group 2 always present in matched pattern")
                .as_str();
            match param {
                "compileViaYul" => {
                    compile_via_yul = value
                        .try_into()
                        .map_err(|error| anyhow::anyhow!("{} on line {}", error, index + 1))?;
                }
                "compileToEwasm" => {}
                "ABIEncoderV1Only" => {
                    abi_encoder_v1_only = value
                        .try_into()
                        .map_err(|error| anyhow::anyhow!("{} on line {}", error, index + 1))?;
                }
                "EVMVersion" => {
                    evm_version = value.try_into().unwrap_or_default();
                }
                "revertStrings" => {
                    revert_strings = value
                        .try_into()
                        .map_err(|error| anyhow::anyhow!("{} on line {}", error, index + 1))?;
                }
                "allowNonExistingFunctions" => {
                    allow_non_existing_functions = value
                        .try_into()
                        .map_err(|error| anyhow::anyhow!("{} on line {}", error, index + 1))?;
                }
                "bytecodeFormat" => {
                    bytecode_format = value.to_owned();
                }
                word => anyhow::bail!(
                    r#"Expected "compileViaYul", "compileToEwasm", "ABIEncoderV1Only", "EVMVersion", "revertStrings", "allowNonExistingFunctions", "bytecodeFormat" on line {}, found: {}"#,
                    index + 1,
                    word
                ),
            }
        }

        Ok(Self {
            compile_via_yul,
            abi_encoder_v1_only,
            evm_version,
            revert_strings,
            allow_non_existing_functions,
            bytecode_format,
        })
    }
}
