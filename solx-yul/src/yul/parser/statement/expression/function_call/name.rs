//!
//! The function name.
//!

///
/// The function name.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum Name {
    /// The user-defined function.
    UserDefined(String),

    /// `x + y`
    Add,
    /// `x - y`
    Sub,
    /// `x * y`
    Mul,
    /// `x / y` or `0` if `y == 0`
    Div,
    /// `x % y` or `0` if `y == 0`
    Mod,
    /// `x / y`, for signed numbers in two’s complement, `0` if `y == 0`
    Sdiv,
    /// `x % y`, for signed numbers in two’s complement, `0` if `y == 0`
    Smod,

    /// `1` if `x < y`, `0` otherwise
    Lt,
    /// `1` if `x > y`, `0` otherwise
    Gt,
    /// `1` if `x == y`, `0` otherwise
    Eq,
    /// `1` if `x == 0`, `0` otherwise
    IsZero,
    /// `1` if `x < y`, `0` otherwise, for signed numbers in two’s complement
    Slt,
    /// `1` if `x > y`, `0` otherwise, for signed numbers in two’s complement
    Sgt,
    /// the number of leading zeros in the binary representation of `x`
    Clz,

    /// bitwise "or" of `x` and `y`
    Or,
    /// bitwise "xor" of `x` and `y`
    Xor,
    /// bitwise "not" of `x` (every bit of `x` is negated)
    Not,
    /// bitwise "and" of `x` and `y`
    And,
    /// logical shift left `y` by `x` bits
    Shl,
    /// logical shift right `y` by `x` bits
    Shr,
    /// signed arithmetic shift right `y` by `x` bits
    Sar,
    /// `n`th byte of `x`, where the most significant byte is the `0`th byte
    Byte,
    /// discard value x
    Pop,

    /// `(x + y) % m` with arbitrary precision arithmetic, `0` if `m == 0`
    AddMod,
    /// `(x * y) % m` with arbitrary precision arithmetic, `0` if `m == 0`
    MulMod,
    /// `x` to the power of `y`
    Exp,
    /// sign extend from `(i*8+7)`th bit counting from least significant
    SignExtend,

    /// `keccak(mem[p…(p+n)))`
    Keccak256,

    /// `mem[p…(p+32))`
    MLoad,
    /// `mem[p…(p+32)) := v`
    MStore,
    /// `mem[p] := v & 0xff` (only modifies a single byte)
    MStore8,
    /// heap memory copy
    MCopy,

    /// `storage[p]`
    SLoad,
    /// `storage[p] := v`
    SStore,
    /// transient `storage[p]`
    TLoad,
    /// transient `storage[p] := v`
    TStore,
    /// `loadimmutable` storage read
    LoadImmutable,
    /// `setimmutable` storage write
    SetImmutable,

    /// call data starting from position `p` (32 bytes)
    CallDataLoad,
    /// size of call data in bytes
    CallDataSize,
    /// copy `s` bytes from calldata at position `f` to memory at position `t`
    CallDataCopy,
    /// size of the code of the current contract / execution context
    CodeSize,
    /// copy `s` bytes from code at position `f` to mem at position `t`
    CodeCopy,
    /// size of the code at address `a`
    ExtCodeSize,
    /// code hash of address `a`
    ExtCodeHash,
    /// size of the last returndata
    ReturnDataSize,
    /// copy `s` bytes from returndata at position `f` to mem at position `t`
    ReturnDataCopy,

    /// end execution, return data `mem[p…(p+s))`
    Return,
    /// end execution, revert state changes, return data `mem[p…(p+s))`
    Revert,
    /// stop execution, identical to `return(0, 0)`
    Stop,
    /// end execution with invalid instruction
    Invalid,

    /// log without topics and data `mem[p…(p+s))`
    Log0,
    /// log with topic t1 and data `mem[p…(p+s))`
    Log1,
    /// log with topics t1, t2 and data `mem[p…(p+s))`
    Log2,
    /// log with topics t1, t2, t3 and data `mem[p…(p+s))`
    Log3,
    /// log with topics t1, t2, t3, t4 and data `mem[p…(p+s))`
    Log4,

    /// call contract at address a with input `mem[in…(in+insize))` providing `g` gas and `v` wei
    /// and output area `mem[out…(out+outsize))` returning 0 on error (e.g. out of gas)
    /// and 1 on success
    /// [See more](https://docs.soliditylang.org/en/v0.8.2/yul.html#yul-call-return-area)
    Call,
    /// identical to call but only use the code from a and stay in the context of the current
    /// contract otherwise
    CallCode,
    /// identical to `callcode` but also keeps `caller` and `callvalue`
    DelegateCall,
    /// identical to `call(g, a, 0, in, insize, out, outsize)` but do not allows state modifications
    StaticCall,

    /// create new contract with code `mem[p…(p+n))` and send `v` wei and return the new address
    ///
    /// Passes bytecode to the system contracts.
    Create,
    /// create new contract with code `mem[p…(p+n))` at address
    /// `keccak256(0xff . this . s . keccak256(mem[p…(p+n)))` and send `v` wei and return the
    /// new address, where `0xff` is a 1-byte value, this is the current contract’s address as a
    /// 20-byte value and `s` is a big-endian 256-bit value
    ///
    /// Passes bytecode to the system contracts.
    Create2,
    /// returns the size in the data area
    DataSize,
    /// is equivalent to `CodeCopy`
    DataCopy,
    /// returns the offset in the data area
    DataOffset,

    /// `linkersymbol` is a stub call
    LinkerSymbol,
    /// `memoryguard` is a stub call
    MemoryGuard,

    /// address of the current contract / execution context
    Address,
    /// call sender (excluding `delegatecall`)
    Caller,

    /// wei sent together with the current call
    CallValue,
    /// gas still available to execution
    Gas,
    /// wei balance at address `a`
    Balance,
    /// equivalent to `balance(address())`, but cheaper
    SelfBalance,

    /// block gas limit of the current block
    GasLimit,
    /// gas price of the transaction
    GasPrice,
    /// transaction sender
    Origin,
    /// ID of the executing chain (EIP 1344)
    ChainId,
    /// current block number
    Number,
    /// timestamp of the current block in seconds since the epoch
    Timestamp,
    /// hash of block nr b - only for last 256 blocks excluding current
    BlockHash,
    /// versioned hash of transaction’s i-th blob
    BlobHash,
    /// difficulty of the current block
    Difficulty,
    /// https://eips.ethereum.org/EIPS/eip-4399
    Prevrandao,
    /// current mining beneficiary
    CoinBase,
    /// current block’s base fee (EIP-3198 and EIP-1559)
    BaseFee,
    /// current block’s blob base fee (EIP-7516 and EIP-4844)
    BlobBaseFee,
    /// size of memory, i.e. largest accessed memory index
    MSize,

    /// Special solx-specific instruction that detects unsafe assembly blocks.
    UnsafeAsm,

    /// verbatim instruction with 0 inputs and 0 outputs
    /// only works in the Yul mode, so it is mostly used as a tool for extending Yul
    Verbatim {
        /// the number of input arguments
        input_size: usize,
        /// the number of output arguments
        output_size: usize,
    },

    /// current position in code
    Pc,
    /// like `codecopy(t, f, s)` but take code at address `a`
    ExtCodeCopy,
    /// end execution, destroy current contract and send funds to `a`
    SelfDestruct,
}

impl From<&str> for Name {
    fn from(input: &str) -> Self {
        match input {
            "add" => Self::Add,
            "sub" => Self::Sub,
            "mul" => Self::Mul,
            "div" => Self::Div,
            "mod" => Self::Mod,
            "sdiv" => Self::Sdiv,
            "smod" => Self::Smod,

            "lt" => Self::Lt,
            "gt" => Self::Gt,
            "eq" => Self::Eq,
            "iszero" => Self::IsZero,
            "slt" => Self::Slt,
            "sgt" => Self::Sgt,
            "clz" => Self::Clz,

            "or" => Self::Or,
            "xor" => Self::Xor,
            "not" => Self::Not,
            "and" => Self::And,
            "shl" => Self::Shl,
            "shr" => Self::Shr,
            "sar" => Self::Sar,
            "byte" => Self::Byte,
            "pop" => Self::Pop,

            "addmod" => Self::AddMod,
            "mulmod" => Self::MulMod,
            "exp" => Self::Exp,
            "signextend" => Self::SignExtend,

            "keccak256" => Self::Keccak256,

            "mload" => Self::MLoad,
            "mstore" => Self::MStore,
            "mstore8" => Self::MStore8,
            "mcopy" => Self::MCopy,

            "sload" => Self::SLoad,
            "sstore" => Self::SStore,
            "tload" => Self::TLoad,
            "tstore" => Self::TStore,
            "loadimmutable" => Self::LoadImmutable,
            "setimmutable" => Self::SetImmutable,

            "calldataload" => Self::CallDataLoad,
            "calldatasize" => Self::CallDataSize,
            "calldatacopy" => Self::CallDataCopy,
            "codesize" => Self::CodeSize,
            "codecopy" => Self::CodeCopy,
            "returndatasize" => Self::ReturnDataSize,
            "returndatacopy" => Self::ReturnDataCopy,
            "extcodesize" => Self::ExtCodeSize,
            "extcodehash" => Self::ExtCodeHash,

            "return" => Self::Return,
            "revert" => Self::Revert,

            "log0" => Self::Log0,
            "log1" => Self::Log1,
            "log2" => Self::Log2,
            "log3" => Self::Log3,
            "log4" => Self::Log4,

            "call" => Self::Call,
            "delegatecall" => Self::DelegateCall,
            "staticcall" => Self::StaticCall,

            "create" => Self::Create,
            "create2" => Self::Create2,
            "datasize" => Self::DataSize,
            "dataoffset" => Self::DataOffset,
            "datacopy" => Self::DataCopy,

            "stop" => Self::Stop,
            "invalid" => Self::Invalid,

            "linkersymbol" => Self::LinkerSymbol,
            "memoryguard" => Self::MemoryGuard,

            "address" => Self::Address,
            "caller" => Self::Caller,

            "callvalue" => Self::CallValue,
            "gas" => Self::Gas,
            "balance" => Self::Balance,
            "selfbalance" => Self::SelfBalance,

            "gaslimit" => Self::GasLimit,
            "gasprice" => Self::GasPrice,
            "origin" => Self::Origin,
            "chainid" => Self::ChainId,
            "timestamp" => Self::Timestamp,
            "number" => Self::Number,
            "blockhash" => Self::BlockHash,
            "blobhash" => Self::BlobHash,
            "difficulty" => Self::Difficulty,
            "prevrandao" => Self::Prevrandao,
            "coinbase" => Self::CoinBase,
            "basefee" => Self::BaseFee,
            "blobbasefee" => Self::BlobBaseFee,
            "msize" => Self::MSize,

            "unsafeasm" => Self::UnsafeAsm,

            "callcode" => Self::CallCode,
            "pc" => Self::Pc,
            "extcodecopy" => Self::ExtCodeCopy,
            "selfdestruct" => Self::SelfDestruct,

            input => Self::UserDefined(input.to_owned()),
        }
    }
}
