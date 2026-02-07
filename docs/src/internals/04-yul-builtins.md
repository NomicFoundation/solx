# Yul Builtins Reference

This chapter lists all Yul builtin functions supported by **solx** and how each is lowered to LLVM IR for the EVM backend.

## Lowering Strategies

Yul builtins are lowered through one of three strategies:

- **Direct LLVM IR**: the builtin maps to native LLVM integer or memory operations on `i256`.
- **LLVM intrinsic**: the builtin maps to an `llvm.evm.*` intrinsic that the EVM backend expands to opcodes.
- **Address space access**: the builtin maps to a load or store in a typed LLVM address space (see [EVM Instructions: Address Space Model](./03-evm-instructions.md#address-space-model)).

## Arithmetic

| Builtin | Lowering | Notes |
|---|---|---|
| `add` | Direct LLVM IR | `add i256` |
| `sub` | Direct LLVM IR | `sub i256` |
| `mul` | Direct LLVM IR | `mul i256` |
| `div` | Direct LLVM IR | Unsigned; returns 0 when divisor is 0 |
| `sdiv` | Direct LLVM IR | Signed; returns 0 when divisor is 0 |
| `mod` | Direct LLVM IR | Unsigned; returns 0 when divisor is 0 |
| `smod` | Direct LLVM IR | Signed; returns 0 when divisor is 0 |
| `addmod` | Intrinsic `llvm.evm.addmod` | `(x + y) % m` without intermediate overflow |
| `mulmod` | Intrinsic `llvm.evm.mulmod` | `(x * y) % m` without intermediate overflow |
| `exp` | Intrinsic `llvm.evm.exp` | Exponentiation |
| `signextend` | Intrinsic `llvm.evm.signextend` | Sign extend from bit `(i*8+7)` |

## Comparison

| Builtin | Lowering | Notes |
|---|---|---|
| `lt` | Direct LLVM IR | Unsigned less-than |
| `gt` | Direct LLVM IR | Unsigned greater-than |
| `slt` | Direct LLVM IR | Signed less-than |
| `sgt` | Direct LLVM IR | Signed greater-than |
| `eq` | Direct LLVM IR | Equality |
| `iszero` | Direct LLVM IR | Check if zero |

## Bitwise

| Builtin | Lowering | Notes |
|---|---|---|
| `and` | Direct LLVM IR | Bitwise AND |
| `or` | Direct LLVM IR | Bitwise OR |
| `xor` | Direct LLVM IR | Bitwise XOR |
| `not` | Direct LLVM IR | Bitwise NOT |
| `shl` | Direct LLVM IR | Shift left; shift >= 256 yields 0 |
| `shr` | Direct LLVM IR | Logical shift right; shift >= 256 yields 0 |
| `sar` | Direct LLVM IR | Arithmetic shift right; shift >= 256 yields sign-extended value |
| `byte` | Intrinsic `llvm.evm.byte` | Extract nth byte |
| `clz` | Intrinsic `llvm.ctlz` | Count leading zeros (requires Osaka EVM version) |

## Hashing

| Builtin | Lowering | Notes |
|---|---|---|
| `keccak256` | Intrinsic `llvm.evm.sha3` | Keccak-256 over memory range |

## Memory

| Builtin | Lowering | Notes |
|---|---|---|
| `mload` | Address space 1 load | Load 32 bytes from heap memory |
| `mstore` | Address space 1 store | Store 32 bytes to heap memory |
| `mstore8` | Intrinsic `llvm.evm.mstore8` | Store single byte to memory |
| `mcopy` | memcpy in address space 1 | EIP-5656 memory copy |
| `msize` | Intrinsic `llvm.evm.msize` | Highest accessed memory index |

## Storage

| Builtin | Lowering | Notes |
|---|---|---|
| `sload` | Address space 5 load | Load from persistent storage |
| `sstore` | Address space 5 store | Store to persistent storage |
| `tload` | Address space 6 load | Load from transient storage (EIP-1153) |
| `tstore` | Address space 6 store | Store to transient storage (EIP-1153) |

## Immutables

| Builtin | Lowering | Notes |
|---|---|---|
| `loadimmutable` | Intrinsic `llvm.evm.loadimmutable` | Load immutable value with metadata identifier |
| `setimmutable` | Special | Set immutable value during deployment |

## Call Data and Return Data

| Builtin | Lowering | Notes |
|---|---|---|
| `calldataload` | Address space 2 load | Load 32 bytes from calldata |
| `calldatasize` | Intrinsic `llvm.evm.calldatasize` | Size of calldata |
| `calldatacopy` | memcpy from address space 2 to 1 | Copy calldata to memory |
| `returndatasize` | Intrinsic `llvm.evm.returndatasize` | Size of return data |
| `returndatacopy` | memcpy from address space 3 to 1 | Copy return data to memory |

## Code Operations

| Builtin | Lowering | Notes |
|---|---|---|
| `codesize` | Intrinsic `llvm.evm.codesize` | Current contract code size |
| `codecopy` | memcpy from address space 4 to 1 | Copy code to memory |
| `extcodesize` | Intrinsic `llvm.evm.extcodesize` | External contract code size |
| `extcodecopy` | Intrinsic `llvm.evm.extcodecopy` | Copy external code to memory |
| `extcodehash` | Intrinsic `llvm.evm.extcodehash` | Hash of external contract code |

## Object and Data Operations

| Builtin | Lowering | Notes |
|---|---|---|
| `datasize` | Intrinsic `llvm.evm.datasize` | Size of a named data object |
| `dataoffset` | Intrinsic `llvm.evm.dataoffset` | Offset of a named data object |
| `datacopy` | Same as `codecopy` | Copy data to memory |

These builtins are used by deploy stubs to reference embedded runtime and dependency objects. See [Binary Layout](./05-binary-layout.md#datasize--dataoffset-builtins) for details.

## Event Logging

| Builtin | Lowering | Notes |
|---|---|---|
| `log0` | Intrinsic `llvm.evm.log0` | Log with 0 topics |
| `log1` | Intrinsic `llvm.evm.log1` | Log with 1 topic |
| `log2` | Intrinsic `llvm.evm.log2` | Log with 2 topics |
| `log3` | Intrinsic `llvm.evm.log3` | Log with 3 topics |
| `log4` | Intrinsic `llvm.evm.log4` | Log with 4 topics |

## Contract Calls

| Builtin | Lowering | Notes |
|---|---|---|
| `call` | Intrinsic `llvm.evm.call` | Call with value transfer |
| `delegatecall` | Intrinsic `llvm.evm.delegatecall` | Call preserving caller and callvalue |
| `staticcall` | Intrinsic `llvm.evm.staticcall` | Read-only call |

Note: `callcode` is rejected at compile time. Use `delegatecall` instead.

## Contract Creation

| Builtin | Lowering | Notes |
|---|---|---|
| `create` | Intrinsic `llvm.evm.create` | Create new contract |
| `create2` | Intrinsic `llvm.evm.create2` | Create at deterministic address |

## Control Flow

| Builtin | Lowering | Notes |
|---|---|---|
| `return` | Intrinsic `llvm.evm.return` | Return data from execution |
| `revert` | Intrinsic `llvm.evm.revert` | Revert with return data |
| `stop` | Intrinsic `llvm.evm.stop` | Stop execution |
| `invalid` | Intrinsic `llvm.evm.invalid` | Invalid instruction (consumes all gas) |

Note: `selfdestruct` is rejected at compile time (deprecated by EIP-6049).

## Block and Transaction Context

| Builtin | Lowering | Notes |
|---|---|---|
| `address` | Intrinsic `llvm.evm.address` | Current contract address |
| `caller` | Intrinsic `llvm.evm.caller` | Message sender |
| `callvalue` | Intrinsic `llvm.evm.callvalue` | Wei sent with call |
| `gas` | Intrinsic `llvm.evm.gas` | Remaining gas |
| `gasprice` | Intrinsic `llvm.evm.gasprice` | Gas price of transaction |
| `balance` | Intrinsic `llvm.evm.balance` | Balance of address |
| `selfbalance` | Intrinsic `llvm.evm.selfbalance` | Current contract balance |
| `origin` | Intrinsic `llvm.evm.origin` | Transaction sender |

## Block Information

| Builtin | Lowering | Notes |
|---|---|---|
| `blockhash` | Intrinsic `llvm.evm.blockhash` | Hash of given block |
| `number` | Intrinsic `llvm.evm.number` | Current block number |
| `timestamp` | Intrinsic `llvm.evm.timestamp` | Block timestamp |
| `coinbase` | Intrinsic `llvm.evm.coinbase` | Block beneficiary |
| `difficulty` | Intrinsic `llvm.evm.difficulty` | Block difficulty (pre-merge) |
| `prevrandao` | Intrinsic `llvm.evm.difficulty` | Previous RANDAO value (EIP-4399, reuses difficulty) |
| `gaslimit` | Intrinsic `llvm.evm.gaslimit` | Block gas limit |
| `chainid` | Intrinsic `llvm.evm.chainid` | Chain ID (EIP-1344) |
| `basefee` | Intrinsic `llvm.evm.basefee` | Base fee per gas (EIP-1559) |

Note: `blobhash` and `blobbasefee` (EIP-4844/EIP-7516) are not yet supported.

## Special and Meta

| Builtin | Lowering | Notes |
|---|---|---|
| `pop` | Optimized away | No code generated |
| `linkersymbol` | Intrinsic `llvm.evm.linkersymbol` | Library linker placeholder |
| `memoryguard` | Special | Reserves a memory region; used by solx to configure the spill area for stack-too-deep mitigation |
