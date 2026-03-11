//!
//! MLIR LLVM dialect operation name constants.
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

// EVM intrinsic function names (used as `callee` attributes in `llvm.call`).

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
