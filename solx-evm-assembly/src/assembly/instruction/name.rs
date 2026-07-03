//!
//! The EVM instruction name.
//!

///
/// The EVM instruction name.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Name {
    /// The eponymous EVM instruction.
    PUSH,
    /// Pushes a constant tag index.
    #[serde(rename = "PUSH [tag]")]
    PUSH_Tag,
    /// Pushes an unknown `data` value.
    #[serde(rename = "PUSH data")]
    PUSH_Data,
    /// Pushes a data size.
    #[serde(rename = "PUSH #[$]")]
    PUSH_DataSize,
    /// Pushes a data offset.
    #[serde(rename = "PUSH [$]")]
    PUSH_DataOffset,

    /// The eponymous EVM instruction.
    PUSH1,
    /// The eponymous EVM instruction.
    PUSH2,
    /// The eponymous EVM instruction.
    PUSH3,
    /// The eponymous EVM instruction.
    PUSH4,
    /// The eponymous EVM instruction.
    PUSH5,
    /// The eponymous EVM instruction.
    PUSH6,
    /// The eponymous EVM instruction.
    PUSH7,
    /// The eponymous EVM instruction.
    PUSH8,
    /// The eponymous EVM instruction.
    PUSH9,
    /// The eponymous EVM instruction.
    PUSH10,
    /// The eponymous EVM instruction.
    PUSH11,
    /// The eponymous EVM instruction.
    PUSH12,
    /// The eponymous EVM instruction.
    PUSH13,
    /// The eponymous EVM instruction.
    PUSH14,
    /// The eponymous EVM instruction.
    PUSH15,
    /// The eponymous EVM instruction.
    PUSH16,
    /// The eponymous EVM instruction.
    PUSH17,
    /// The eponymous EVM instruction.
    PUSH18,
    /// The eponymous EVM instruction.
    PUSH19,
    /// The eponymous EVM instruction.
    PUSH20,
    /// The eponymous EVM instruction.
    PUSH21,
    /// The eponymous EVM instruction.
    PUSH22,
    /// The eponymous EVM instruction.
    PUSH23,
    /// The eponymous EVM instruction.
    PUSH24,
    /// The eponymous EVM instruction.
    PUSH25,
    /// The eponymous EVM instruction.
    PUSH26,
    /// The eponymous EVM instruction.
    PUSH27,
    /// The eponymous EVM instruction.
    PUSH28,
    /// The eponymous EVM instruction.
    PUSH29,
    /// The eponymous EVM instruction.
    PUSH30,
    /// The eponymous EVM instruction.
    PUSH31,
    /// The eponymous EVM instruction.
    PUSH32,
    /// The eponymous EVM instruction.
    PUSH0,

    /// The eponymous EVM instruction.
    DUP1,
    /// The eponymous EVM instruction.
    DUP2,
    /// The eponymous EVM instruction.
    DUP3,
    /// The eponymous EVM instruction.
    DUP4,
    /// The eponymous EVM instruction.
    DUP5,
    /// The eponymous EVM instruction.
    DUP6,
    /// The eponymous EVM instruction.
    DUP7,
    /// The eponymous EVM instruction.
    DUP8,
    /// The eponymous EVM instruction.
    DUP9,
    /// The eponymous EVM instruction.
    DUP10,
    /// The eponymous EVM instruction.
    DUP11,
    /// The eponymous EVM instruction.
    DUP12,
    /// The eponymous EVM instruction.
    DUP13,
    /// The eponymous EVM instruction.
    DUP14,
    /// The eponymous EVM instruction.
    DUP15,
    /// The eponymous EVM instruction.
    DUP16,
    /// The eponymous EVM instruction.
    DUPX,

    /// The eponymous EVM instruction.
    SWAP1,
    /// The eponymous EVM instruction.
    SWAP2,
    /// The eponymous EVM instruction.
    SWAP3,
    /// The eponymous EVM instruction.
    SWAP4,
    /// The eponymous EVM instruction.
    SWAP5,
    /// The eponymous EVM instruction.
    SWAP6,
    /// The eponymous EVM instruction.
    SWAP7,
    /// The eponymous EVM instruction.
    SWAP8,
    /// The eponymous EVM instruction.
    SWAP9,
    /// The eponymous EVM instruction.
    SWAP10,
    /// The eponymous EVM instruction.
    SWAP11,
    /// The eponymous EVM instruction.
    SWAP12,
    /// The eponymous EVM instruction.
    SWAP13,
    /// The eponymous EVM instruction.
    SWAP14,
    /// The eponymous EVM instruction.
    SWAP15,
    /// The eponymous EVM instruction.
    SWAP16,
    /// The eponymous EVM instruction.
    SWAPX,

    /// The eponymous EVM instruction.
    POP,

    /// Sets the current basic code block.
    #[serde(rename = "tag")]
    Tag,
    /// The eponymous EVM instruction.
    JUMP,
    /// The eponymous EVM instruction.
    JUMPI,
    /// The eponymous EVM instruction.
    JUMPDEST,

    /// The eponymous EVM instruction.
    ADD,
    /// The eponymous EVM instruction.
    SUB,
    /// The eponymous EVM instruction.
    MUL,
    /// The eponymous EVM instruction.
    DIV,
    /// The eponymous EVM instruction.
    MOD,
    /// The eponymous EVM instruction.
    SDIV,
    /// The eponymous EVM instruction.
    SMOD,

    /// The eponymous EVM instruction.
    LT,
    /// The eponymous EVM instruction.
    GT,
    /// The eponymous EVM instruction.
    EQ,
    /// The eponymous EVM instruction.
    ISZERO,
    /// The eponymous EVM instruction.
    SLT,
    /// The eponymous EVM instruction.
    SGT,

    /// The eponymous EVM instruction.
    OR,
    /// The eponymous EVM instruction.
    XOR,
    /// The eponymous EVM instruction.
    NOT,
    /// The eponymous EVM instruction.
    AND,
    /// The eponymous EVM instruction.
    SHL,
    /// The eponymous EVM instruction.
    SHR,
    /// The eponymous EVM instruction.
    SAR,
    /// The eponymous EVM instruction.
    CLZ,
    /// The eponymous EVM instruction.
    BYTE,

    /// The eponymous EVM instruction.
    ADDMOD,
    /// The eponymous EVM instruction.
    MULMOD,
    /// The eponymous EVM instruction.
    EXP,
    /// The eponymous EVM instruction.
    SIGNEXTEND,
    /// The eponymous EVM instruction.
    SHA3,
    /// The eponymous EVM instruction.
    KECCAK256,

    /// The eponymous EVM instruction.
    MLOAD,
    /// The eponymous EVM instruction.
    MSTORE,
    /// The eponymous EVM instruction.
    MSTORE8,
    /// The eponymous EVM instruction.
    MCOPY,

    /// The eponymous EVM instruction.
    SLOAD,
    /// The eponymous EVM instruction.
    SSTORE,
    /// The eponymous EVM instruction.
    TLOAD,
    /// The eponymous EVM instruction.
    TSTORE,
    /// The eponymous EVM instruction.
    PUSHIMMUTABLE,
    /// The eponymous EVM instruction.
    ASSIGNIMMUTABLE,

    /// The eponymous EVM instruction.
    CALLDATALOAD,
    /// The eponymous EVM instruction.
    CALLDATASIZE,
    /// The eponymous EVM instruction.
    CALLDATACOPY,
    /// The eponymous EVM instruction.
    CODESIZE,
    /// The eponymous EVM instruction.
    CODECOPY,
    /// The eponymous EVM instruction.
    PUSHSIZE,
    /// The eponymous EVM instruction.
    EXTCODESIZE,
    /// The eponymous EVM instruction.
    EXTCODEHASH,
    /// The eponymous EVM instruction.
    RETURNDATASIZE,
    /// The eponymous EVM instruction.
    RETURNDATACOPY,

    /// The eponymous EVM instruction.
    RETURN,
    /// The eponymous EVM instruction.
    REVERT,
    /// The eponymous EVM instruction.
    STOP,
    /// The eponymous EVM instruction.
    INVALID,

    /// The eponymous EVM instruction.
    LOG0,
    /// The eponymous EVM instruction.
    LOG1,
    /// The eponymous EVM instruction.
    LOG2,
    /// The eponymous EVM instruction.
    LOG3,
    /// The eponymous EVM instruction.
    LOG4,

    /// The eponymous EVM instruction.
    CALL,
    /// The eponymous EVM instruction.
    STATICCALL,
    /// The eponymous EVM instruction.
    DELEGATECALL,

    /// The eponymous EVM instruction.
    CREATE,
    /// The eponymous EVM instruction.
    CREATE2,

    /// The eponymous EVM instruction.
    ADDRESS,
    /// The eponymous EVM instruction.
    CALLER,

    /// The eponymous EVM instruction.
    CALLVALUE,
    /// The eponymous EVM instruction.
    GAS,
    /// The eponymous EVM instruction.
    BALANCE,
    /// The eponymous EVM instruction.
    SELFBALANCE,

    /// The eponymous EVM instruction.
    PUSHLIB,
    /// The eponymous EVM instruction.
    PUSHDEPLOYADDRESS,
    /// The eponymous EVM instruction.
    MEMORYGUARD,

    /// The eponymous EVM instruction.
    GASLIMIT,
    /// The eponymous EVM instruction.
    GASPRICE,
    /// The eponymous EVM instruction.
    ORIGIN,
    /// The eponymous EVM instruction.
    CHAINID,
    /// The eponymous EVM instruction.
    TIMESTAMP,
    /// The eponymous EVM instruction.
    NUMBER,
    /// The eponymous EVM instruction.
    BLOCKHASH,
    /// The eponymous EVM instruction.
    BLOBHASH,
    /// The eponymous EVM instruction.
    DIFFICULTY,
    /// The eponymous EVM instruction.
    PREVRANDAO,
    /// The eponymous EVM instruction.
    COINBASE,
    /// The eponymous EVM instruction.
    BASEFEE,
    /// The eponymous EVM instruction.
    BLOBBASEFEE,
    /// The eponymous EVM instruction.
    MSIZE,

    /// The eponymous EVM instruction.
    CALLCODE,
    /// The eponymous EVM instruction.
    PC,
    /// The eponymous EVM instruction.
    EXTCODECOPY,
    /// The eponymous EVM instruction.
    SELFDESTRUCT,

    /// Special solx-specific instruction that detects unsafe assembly blocks.
    UNSAFEASM,

    /// The defined function call instruction.
    #[serde(skip)]
    RecursiveCall {
        /// The called function name.
        name: String,
        /// The called function key.
        entry_key: solx_codegen_evm::BlockKey,
        /// The stack state hash after return.
        stack_hash: u64,
        /// The input size.
        input_size: usize,
        /// The output size.
        output_size: usize,
        /// The return address.
        return_address: solx_codegen_evm::BlockKey,
    },
    /// The defined function return instruction.
    #[serde(skip)]
    RecursiveReturn {
        /// The output size.
        input_size: usize,
    },
}

impl Name {
    ///
    /// Returns the static mnemonic of the instruction.
    ///
    pub const fn mnemonic(&self) -> &'static str {
        match self {
            Self::PUSH => "PUSH",
            Self::PUSH_Tag => "PUSH_Tag",
            Self::PUSH_Data => "PUSH_Data",
            Self::PUSH_DataSize => "PUSH_DataSize",
            Self::PUSH_DataOffset => "PUSH_DataOffset",

            Self::PUSH1 => "PUSH1",
            Self::PUSH2 => "PUSH2",
            Self::PUSH3 => "PUSH3",
            Self::PUSH4 => "PUSH4",
            Self::PUSH5 => "PUSH5",
            Self::PUSH6 => "PUSH6",
            Self::PUSH7 => "PUSH7",
            Self::PUSH8 => "PUSH8",
            Self::PUSH9 => "PUSH9",
            Self::PUSH10 => "PUSH10",
            Self::PUSH11 => "PUSH11",
            Self::PUSH12 => "PUSH12",
            Self::PUSH13 => "PUSH13",
            Self::PUSH14 => "PUSH14",
            Self::PUSH15 => "PUSH15",
            Self::PUSH16 => "PUSH16",
            Self::PUSH17 => "PUSH17",
            Self::PUSH18 => "PUSH18",
            Self::PUSH19 => "PUSH19",
            Self::PUSH20 => "PUSH20",
            Self::PUSH21 => "PUSH21",
            Self::PUSH22 => "PUSH22",
            Self::PUSH23 => "PUSH23",
            Self::PUSH24 => "PUSH24",
            Self::PUSH25 => "PUSH25",
            Self::PUSH26 => "PUSH26",
            Self::PUSH27 => "PUSH27",
            Self::PUSH28 => "PUSH28",
            Self::PUSH29 => "PUSH29",
            Self::PUSH30 => "PUSH30",
            Self::PUSH31 => "PUSH31",
            Self::PUSH32 => "PUSH32",
            Self::PUSH0 => "PUSH0",

            Self::DUP1 => "DUP1",
            Self::DUP2 => "DUP2",
            Self::DUP3 => "DUP3",
            Self::DUP4 => "DUP4",
            Self::DUP5 => "DUP5",
            Self::DUP6 => "DUP6",
            Self::DUP7 => "DUP7",
            Self::DUP8 => "DUP8",
            Self::DUP9 => "DUP9",
            Self::DUP10 => "DUP10",
            Self::DUP11 => "DUP11",
            Self::DUP12 => "DUP12",
            Self::DUP13 => "DUP13",
            Self::DUP14 => "DUP14",
            Self::DUP15 => "DUP15",
            Self::DUP16 => "DUP16",
            Self::DUPX => "DUPX",

            Self::SWAP1 => "SWAP1",
            Self::SWAP2 => "SWAP2",
            Self::SWAP3 => "SWAP3",
            Self::SWAP4 => "SWAP4",
            Self::SWAP5 => "SWAP5",
            Self::SWAP6 => "SWAP6",
            Self::SWAP7 => "SWAP7",
            Self::SWAP8 => "SWAP8",
            Self::SWAP9 => "SWAP9",
            Self::SWAP10 => "SWAP10",
            Self::SWAP11 => "SWAP11",
            Self::SWAP12 => "SWAP12",
            Self::SWAP13 => "SWAP13",
            Self::SWAP14 => "SWAP14",
            Self::SWAP15 => "SWAP15",
            Self::SWAP16 => "SWAP16",
            Self::SWAPX => "SWAPX",

            Self::POP => "POP",

            Self::Tag => "Tag",
            Self::JUMP => "JUMP",
            Self::JUMPI => "JUMPI",
            Self::JUMPDEST => "JUMPDEST",

            Self::ADD => "ADD",
            Self::SUB => "SUB",
            Self::MUL => "MUL",
            Self::DIV => "DIV",
            Self::MOD => "MOD",
            Self::SDIV => "SDIV",
            Self::SMOD => "SMOD",

            Self::LT => "LT",
            Self::GT => "GT",
            Self::EQ => "EQ",
            Self::ISZERO => "ISZERO",
            Self::SLT => "SLT",
            Self::SGT => "SGT",

            Self::OR => "OR",
            Self::XOR => "XOR",
            Self::NOT => "NOT",
            Self::AND => "AND",
            Self::SHL => "SHL",
            Self::SHR => "SHR",
            Self::SAR => "SAR",
            Self::CLZ => "CLZ",
            Self::BYTE => "BYTE",

            Self::ADDMOD => "ADDMOD",
            Self::MULMOD => "MULMOD",
            Self::EXP => "EXP",
            Self::SIGNEXTEND => "SIGNEXTEND",
            Self::SHA3 => "SHA3",
            Self::KECCAK256 => "KECCAK256",

            Self::MLOAD => "MLOAD",
            Self::MSTORE => "MSTORE",
            Self::MSTORE8 => "MSTORE8",
            Self::MCOPY => "MCOPY",

            Self::SLOAD => "SLOAD",
            Self::SSTORE => "SSTORE",
            Self::TLOAD => "TLOAD",
            Self::TSTORE => "TSTORE",
            Self::PUSHIMMUTABLE => "PUSHIMMUTABLE",
            Self::ASSIGNIMMUTABLE => "ASSIGNIMMUTABLE",

            Self::CALLDATALOAD => "CALLDATALOAD",
            Self::CALLDATASIZE => "CALLDATASIZE",
            Self::CALLDATACOPY => "CALLDATACOPY",
            Self::CODESIZE => "CODESIZE",
            Self::CODECOPY => "CODECOPY",
            Self::PUSHSIZE => "PUSHSIZE",
            Self::EXTCODESIZE => "EXTCODESIZE",
            Self::EXTCODEHASH => "EXTCODEHASH",
            Self::RETURNDATASIZE => "RETURNDATASIZE",
            Self::RETURNDATACOPY => "RETURNDATACOPY",

            Self::RETURN => "RETURN",
            Self::REVERT => "REVERT",
            Self::STOP => "STOP",
            Self::INVALID => "INVALID",

            Self::LOG0 => "LOG0",
            Self::LOG1 => "LOG1",
            Self::LOG2 => "LOG2",
            Self::LOG3 => "LOG3",
            Self::LOG4 => "LOG4",

            Self::CALL => "CALL",
            Self::STATICCALL => "STATICCALL",
            Self::DELEGATECALL => "DELEGATECALL",

            Self::CREATE => "CREATE",
            Self::CREATE2 => "CREATE2",

            Self::ADDRESS => "ADDRESS",
            Self::CALLER => "CALLER",

            Self::CALLVALUE => "CALLVALUE",
            Self::GAS => "GAS",
            Self::BALANCE => "BALANCE",
            Self::SELFBALANCE => "SELFBALANCE",

            Self::PUSHLIB => "PUSHLIB",
            Self::PUSHDEPLOYADDRESS => "PUSHDEPLOYADDRESS",
            Self::MEMORYGUARD => "MEMORYGUARD",

            Self::GASLIMIT => "GASLIMIT",
            Self::GASPRICE => "GASPRICE",
            Self::ORIGIN => "ORIGIN",
            Self::CHAINID => "CHAINID",
            Self::TIMESTAMP => "TIMESTAMP",
            Self::NUMBER => "NUMBER",
            Self::BLOCKHASH => "BLOCKHASH",
            Self::BLOBHASH => "BLOBHASH",
            Self::DIFFICULTY => "DIFFICULTY",
            Self::PREVRANDAO => "PREVRANDAO",
            Self::COINBASE => "COINBASE",
            Self::BASEFEE => "BASEFEE",
            Self::BLOBBASEFEE => "BLOBBASEFEE",
            Self::MSIZE => "MSIZE",

            Self::CALLCODE => "CALLCODE",
            Self::PC => "PC",
            Self::EXTCODECOPY => "EXTCODECOPY",
            Self::SELFDESTRUCT => "SELFDESTRUCT",

            Self::UNSAFEASM => "UNSAFEASM",

            Self::RecursiveCall { .. } => "RECURSIVE_CALL",
            Self::RecursiveReturn { .. } => "RECURSIVE_RETURN",
        }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecursiveCall {
                name,
                entry_key,
                input_size,
                output_size,
                return_address,
                ..
            } => write!(
                f,
                "RECURSIVE_CALL({name}_{entry_key}, {input_size}, {output_size}, {return_address})",
            ),
            Self::RecursiveReturn { input_size } => write!(f, "RECURSIVE_RETURN({input_size})"),
            _ => f.write_str(self.mnemonic()),
        }
    }
}
