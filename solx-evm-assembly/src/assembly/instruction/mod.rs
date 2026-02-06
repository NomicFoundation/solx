//!
//! The EVM instruction.
//!

pub mod codecopy;
pub mod jump;
pub mod name;
pub mod stack;

use std::collections::BTreeMap;

use self::name::Name;

///
/// The EVM instruction.
///
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Instruction {
    /// The opcode or tag identifier.
    pub name: Name,
    /// The optional value argument.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The source code identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<isize>,
    /// The source code location begin.
    pub begin: isize,
    /// The source code location end.
    pub end: isize,
}

impl Instruction {
    ///
    /// Returns the number of input stack arguments.
    ///
    pub const fn input_size(&self, version: &semver::Version) -> usize {
        match self.name {
            Name::POP => 1,

            Name::JUMP => 1,
            Name::JUMPI => 2,

            Name::DUPX => 1,
            Name::SWAPX => 1,
            Name::MEMORYGUARD => 1,

            Name::ADD => 2,
            Name::SUB => 2,
            Name::MUL => 2,
            Name::DIV => 2,
            Name::MOD => 2,
            Name::SDIV => 2,
            Name::SMOD => 2,

            Name::LT => 2,
            Name::GT => 2,
            Name::EQ => 2,
            Name::ISZERO => 1,
            Name::SLT => 2,
            Name::SGT => 2,

            Name::OR => 2,
            Name::XOR => 2,
            Name::NOT => 1,
            Name::AND => 2,
            Name::SHL => 2,
            Name::SHR => 2,
            Name::SAR => 2,
            Name::CLZ => 1,
            Name::BYTE => 2,

            Name::ADDMOD => 3,
            Name::MULMOD => 3,
            Name::EXP => 2,
            Name::SIGNEXTEND => 2,
            Name::SHA3 => 2,
            Name::KECCAK256 => 2,

            Name::MLOAD => 1,
            Name::MSTORE => 2,
            Name::MSTORE8 => 2,
            Name::MCOPY => 3,

            Name::SLOAD => 1,
            Name::SSTORE => 2,
            Name::TLOAD => 1,
            Name::TSTORE => 2,
            Name::PUSHIMMUTABLE => 0,
            Name::ASSIGNIMMUTABLE => {
                if version.minor >= 8 {
                    2
                } else {
                    1
                }
            }

            Name::CALLDATALOAD => 1,
            Name::CALLDATACOPY => 3,
            Name::CODECOPY => 3,
            Name::RETURNDATACOPY => 3,
            Name::EXTCODESIZE => 1,
            Name::EXTCODEHASH => 1,

            Name::CALL => 7,
            Name::CALLCODE => 7,
            Name::STATICCALL => 6,
            Name::DELEGATECALL => 6,

            Name::RETURN => 2,
            Name::REVERT => 2,
            Name::SELFDESTRUCT => 1,

            Name::LOG0 => 2,
            Name::LOG1 => 3,
            Name::LOG2 => 4,
            Name::LOG3 => 5,
            Name::LOG4 => 6,

            Name::CREATE => 3,
            Name::CREATE2 => 4,

            Name::BALANCE => 1,

            Name::BLOCKHASH => 1,
            Name::BLOBHASH => 1,

            Name::EXTCODECOPY => 4,

            Name::RecursiveCall { input_size, .. } => input_size,
            Name::RecursiveReturn { input_size } => input_size,

            _ => 0,
        }
    }

    ///
    /// Returns the number of output stack arguments.
    ///
    pub const fn output_size(&self) -> usize {
        match self.name {
            Name::PUSH => 1,
            Name::PUSH_Data => 1,
            Name::PUSH_Tag => 1,
            Name::PUSH_DataOffset => 1,
            Name::PUSH_DataSize => 1,
            Name::PUSHLIB => 1,
            Name::PUSHDEPLOYADDRESS => 1,
            Name::MEMORYGUARD => 1,

            Name::PUSH1 => 1,
            Name::PUSH2 => 1,
            Name::PUSH3 => 1,
            Name::PUSH4 => 1,
            Name::PUSH5 => 1,
            Name::PUSH6 => 1,
            Name::PUSH7 => 1,
            Name::PUSH8 => 1,
            Name::PUSH9 => 1,
            Name::PUSH10 => 1,
            Name::PUSH11 => 1,
            Name::PUSH12 => 1,
            Name::PUSH13 => 1,
            Name::PUSH14 => 1,
            Name::PUSH15 => 1,
            Name::PUSH16 => 1,
            Name::PUSH17 => 1,
            Name::PUSH18 => 1,
            Name::PUSH19 => 1,
            Name::PUSH20 => 1,
            Name::PUSH21 => 1,
            Name::PUSH22 => 1,
            Name::PUSH23 => 1,
            Name::PUSH24 => 1,
            Name::PUSH25 => 1,
            Name::PUSH26 => 1,
            Name::PUSH27 => 1,
            Name::PUSH28 => 1,
            Name::PUSH29 => 1,
            Name::PUSH30 => 1,
            Name::PUSH31 => 1,
            Name::PUSH32 => 1,
            Name::PUSH0 => 1,

            Name::DUP1 => 1,
            Name::DUP2 => 1,
            Name::DUP3 => 1,
            Name::DUP4 => 1,
            Name::DUP5 => 1,
            Name::DUP6 => 1,
            Name::DUP7 => 1,
            Name::DUP8 => 1,
            Name::DUP9 => 1,
            Name::DUP10 => 1,
            Name::DUP11 => 1,
            Name::DUP12 => 1,
            Name::DUP13 => 1,
            Name::DUP14 => 1,
            Name::DUP15 => 1,
            Name::DUP16 => 1,
            Name::DUPX => 1,

            Name::ADD => 1,
            Name::SUB => 1,
            Name::MUL => 1,
            Name::DIV => 1,
            Name::MOD => 1,
            Name::SDIV => 1,
            Name::SMOD => 1,

            Name::LT => 1,
            Name::GT => 1,
            Name::EQ => 1,
            Name::ISZERO => 1,
            Name::SLT => 1,
            Name::SGT => 1,

            Name::OR => 1,
            Name::XOR => 1,
            Name::NOT => 1,
            Name::AND => 1,
            Name::SHL => 1,
            Name::SHR => 1,
            Name::SAR => 1,
            Name::CLZ => 1,
            Name::BYTE => 1,

            Name::ADDMOD => 1,
            Name::MULMOD => 1,
            Name::EXP => 1,
            Name::SIGNEXTEND => 1,
            Name::SHA3 => 1,
            Name::KECCAK256 => 1,

            Name::MLOAD => 1,

            Name::SLOAD => 1,
            Name::TLOAD => 1,
            Name::PUSHIMMUTABLE => 1,

            Name::CALLDATALOAD => 1,
            Name::CALLDATASIZE => 1,
            Name::CODESIZE => 1,
            Name::PUSHSIZE => 1,
            Name::RETURNDATASIZE => 1,
            Name::EXTCODESIZE => 1,
            Name::EXTCODEHASH => 1,

            Name::CALL => 1,
            Name::CALLCODE => 1,
            Name::STATICCALL => 1,
            Name::DELEGATECALL => 1,

            Name::CREATE => 1,
            Name::CREATE2 => 1,

            Name::ADDRESS => 1,
            Name::CALLER => 1,
            Name::TIMESTAMP => 1,
            Name::NUMBER => 1,

            Name::CALLVALUE => 1,
            Name::GAS => 1,
            Name::BALANCE => 1,
            Name::SELFBALANCE => 1,

            Name::GASLIMIT => 1,
            Name::GASPRICE => 1,
            Name::ORIGIN => 1,
            Name::CHAINID => 1,
            Name::BLOCKHASH => 1,
            Name::BLOBHASH => 1,
            Name::DIFFICULTY => 1,
            Name::PREVRANDAO => 1,
            Name::COINBASE => 1,
            Name::MSIZE => 1,

            Name::BASEFEE => 1,
            Name::BLOBBASEFEE => 1,
            Name::PC => 1,

            Name::RecursiveCall { output_size, .. } => output_size,

            _ => 0,
        }
    }

    ///
    /// Replaces the instruction data aliases with the actual data.
    ///
    pub fn replace_data_aliases(
        instructions: &mut [Self],
        mapping: &BTreeMap<String, String>,
    ) -> anyhow::Result<()> {
        for instruction in instructions.iter_mut() {
            match instruction {
                Instruction {
                    name: Name::PUSH_DataOffset | Name::PUSH_DataSize,
                    value: Some(value),
                    ..
                } => {
                    *value = mapping.get(value.as_str()).cloned().ok_or_else(|| {
                        anyhow::anyhow!("Contract alias `{value}` data not found")
                    })?;
                }
                Instruction {
                    name: Name::PUSH_Data,
                    value: Some(value),
                    ..
                } => {
                    let mut key_extended =
                        "0".repeat(solx_utils::BYTE_LENGTH_FIELD * 2 - value.len());
                    key_extended.push_str(value.as_str());

                    *value = mapping.get(key_extended.as_str()).cloned().ok_or_else(|| {
                        anyhow::anyhow!("Data chunk alias `{key_extended}` data not found")
                    })?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    ///
    /// Initializes an `INVALID` instruction to terminate an invalid unreachable block part.
    ///
    pub fn invalid(previous: &Self) -> Self {
        Self {
            name: Name::INVALID,
            value: None,

            source: previous.source,
            begin: previous.begin,
            end: previous.end,
        }
    }

    ///
    /// Initializes a function `Call` instruction.
    ///
    pub fn call(
        name: String,
        entry_key: solx_codegen_evm::BlockKey,
        stack_hash: u64,
        input_size: usize,
        output_size: usize,
        return_address: solx_codegen_evm::BlockKey,
        previous: &Self,
    ) -> Self {
        Self {
            name: Name::RecursiveCall {
                name,
                entry_key,
                stack_hash,
                input_size,
                output_size,
                return_address,
            },
            value: None,

            source: previous.source,
            begin: previous.begin,
            end: previous.end,
        }
    }

    ///
    /// Initializes a `Return` instruction.
    ///
    pub fn r#return(input_size: usize, previous: &Self) -> Self {
        Self {
            name: Name::RecursiveReturn { input_size },
            value: None,

            source: previous.source,
            begin: previous.begin,
            end: previous.end,
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.to_string();
        match self.name {
            Name::Tag => write!(f, "{name:4}"),
            _ => write!(f, "{name:20}"),
        }?;
        match self.value.as_ref() {
            Some(value) if value.len() <= 64 => write!(f, "{value}")?,
            Some(value) => write!(f, "... {}", &value[value.len() - 60..])?,
            None => {}
        }
        Ok(())
    }
}
