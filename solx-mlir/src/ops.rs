//!
//! MLIR operation name constants for LLVM and Sol dialects.
//!
//! Centralizes the string literals used with `OperationBuilder::new()` to
//! prevent typos and make operation references greppable.
//!

/// `llvm.mlir.constant` — materializes a compile-time constant.
pub const MLIR_CONSTANT: &str = "llvm.mlir.constant";
/// `llvm.return` — returns from a function.
pub const RETURN: &str = "llvm.return";
/// `llvm.br` — unconditional branch.
pub const BR: &str = "llvm.br";
/// `llvm.cond_br` — conditional branch.
pub const COND_BR: &str = "llvm.cond_br";
/// `llvm.icmp` — integer comparison.
pub const ICMP: &str = "llvm.icmp";
/// `llvm.zext` — zero extension.
pub const ZEXT: &str = "llvm.zext";
/// `llvm.alloca` — stack allocation.
pub const ALLOCA: &str = "llvm.alloca";
/// `llvm.inttoptr` — integer to pointer cast.
pub const INTTOPTR: &str = "llvm.inttoptr";
/// `llvm.call` — function call.
pub const CALL: &str = "llvm.call";
/// `llvm.add` — integer addition.
pub const ADD: &str = "llvm.add";
/// `llvm.sub` — integer subtraction.
pub const SUB: &str = "llvm.sub";
/// `llvm.mul` — integer multiplication.
pub const MUL: &str = "llvm.mul";
/// `llvm.udiv` — unsigned integer division.
pub const UDIV: &str = "llvm.udiv";
/// `llvm.urem` — unsigned integer remainder.
pub const UREM: &str = "llvm.urem";
/// `llvm.and` — bitwise AND.
pub const AND: &str = "llvm.and";
/// `llvm.or` — bitwise OR.
pub const OR: &str = "llvm.or";
/// `llvm.xor` — bitwise XOR.
pub const XOR: &str = "llvm.xor";
/// `llvm.shl` — shift left.
pub const SHL: &str = "llvm.shl";
/// `llvm.lshr` — logical shift right.
pub const LSHR: &str = "llvm.lshr";

// EVM intrinsic MLIR operation names (emitted as direct operations via OperationBuilder).

/// `llvm.evm.return` — halt execution and return data.
pub const EVM_RETURN: &str = "llvm.evm.return";
/// `llvm.evm.revert` — halt execution and revert state.
pub const EVM_REVERT: &str = "llvm.evm.revert";
/// `llvm.evm.calldataload` — load 32 bytes from calldata.
pub const EVM_CALLDATALOAD: &str = "llvm.evm.calldataload";
/// `llvm.evm.origin` — get execution originator.
pub const EVM_ORIGIN: &str = "llvm.evm.origin";
/// `llvm.evm.gasprice` — get gas price.
pub const EVM_GASPRICE: &str = "llvm.evm.gasprice";
/// `llvm.evm.caller` — get caller address.
pub const EVM_CALLER: &str = "llvm.evm.caller";
/// `llvm.evm.callvalue` — get deposited value.
pub const EVM_CALLVALUE: &str = "llvm.evm.callvalue";
/// `llvm.evm.timestamp` — get block timestamp.
pub const EVM_TIMESTAMP: &str = "llvm.evm.timestamp";
/// `llvm.evm.number` — get block number.
pub const EVM_NUMBER: &str = "llvm.evm.number";
/// `llvm.evm.coinbase` — get block coinbase.
pub const EVM_COINBASE: &str = "llvm.evm.coinbase";
/// `llvm.evm.chainid` — get chain ID.
pub const EVM_CHAINID: &str = "llvm.evm.chainid";
/// `llvm.evm.basefee` — get block base fee.
pub const EVM_BASEFEE: &str = "llvm.evm.basefee";
/// `llvm.evm.gaslimit` — get block gas limit.
pub const EVM_GASLIMIT: &str = "llvm.evm.gaslimit";
/// `llvm.evm.call` — message call into an account.
pub const EVM_CALL: &str = "llvm.evm.call";

// Sol dialect operation names.

/// Sol dialect operation name constants.
pub mod sol {
    /// `sol.contract` — contract symbol table container.
    pub const CONTRACT: &str = "sol.contract";
    /// `sol.func` — function definition with selector and mutability.
    pub const FUNC: &str = "sol.func";
    /// `sol.constant` — compile-time constant.
    pub const CONSTANT: &str = "sol.constant";
    /// `sol.return` — return from function.
    pub const RETURN: &str = "sol.return";
    /// `sol.yield` — region terminator for structured control flow.
    pub const YIELD: &str = "sol.yield";
    /// `sol.condition` — loop condition terminator.
    pub const CONDITION: &str = "sol.condition";
    /// `sol.alloca` — stack allocation.
    pub const ALLOCA: &str = "sol.alloca";
    /// `sol.load` — load from pointer.
    pub const LOAD: &str = "sol.load";
    /// `sol.store` — store to pointer.
    pub const STORE: &str = "sol.store";
    /// `sol.call` — function call.
    pub const CALL: &str = "sol.call";
    /// `sol.if` — structured if/else.
    pub const IF: &str = "sol.if";
    /// `sol.while` — structured while loop.
    pub const WHILE: &str = "sol.while";
    /// `sol.for` — structured for loop.
    pub const FOR: &str = "sol.for";
    /// `sol.add` — unchecked addition.
    pub const ADD: &str = "sol.add";
    /// `sol.sub` — unchecked subtraction.
    pub const SUB: &str = "sol.sub";
    /// `sol.mul` — unchecked multiplication.
    pub const MUL: &str = "sol.mul";
    /// `sol.div` — unchecked division.
    pub const DIV: &str = "sol.div";
    /// `sol.mod` — unchecked modulo.
    pub const MOD: &str = "sol.mod";
    /// `sol.cadd` — checked addition.
    pub const CADD: &str = "sol.cadd";
    /// `sol.csub` — checked subtraction.
    pub const CSUB: &str = "sol.csub";
    /// `sol.cmul` — checked multiplication.
    pub const CMUL: &str = "sol.cmul";
    /// `sol.cmp` — comparison.
    pub const CMP: &str = "sol.cmp";
    /// `sol.cast` — type cast.
    pub const CAST: &str = "sol.cast";
    /// `sol.state_var` — state variable declaration.
    pub const STATE_VAR: &str = "sol.state_var";
    /// `sol.and` — bitwise AND.
    pub const AND: &str = "sol.and";
    /// `sol.or` — bitwise OR.
    pub const OR: &str = "sol.or";
    /// `sol.xor` — bitwise XOR.
    pub const XOR: &str = "sol.xor";
    /// `sol.shl` — shift left.
    pub const SHL: &str = "sol.shl";
    /// `sol.shr` — shift right.
    pub const SHR: &str = "sol.shr";
}
