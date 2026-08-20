// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*arithmetic.*}}
// CHECK:   sol.inline_asm {
// A Yul argument list is materialized in an order the two frontends disagree on, so this
// file asserts which ops a builtin lowers to; assembly_evaluation_order.sol asserts the order.
// CHECK-DAG:     sol.yul_ptr_cast %{{.*}} : !sol.ptr<ui256, Stack> -> !yul.ptr
// CHECK-DAG:     yul.load %{{.*}} : !yul.ptr -> i256
// CHECK-DAG:     yul.constant 1
// CHECK:     %[[SUM:.*]] = yul.add
// CHECK:     %[[SLOT:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[SUM]], %[[SLOT]] : i256, !yul.ptr
// CHECK:     yul.mul
// CHECK:     yul.sub
// CHECK:     yul.div
// CHECK:     yul.sdiv
// CHECK:     yul.mod
// CHECK:     yul.smod
// CHECK:     yul.exp
// CHECK:     yul.addmod
// CHECK:     yul.mulmod
// CHECK:     yul.signextend

// CHECK: sol.func @{{.*bitwise.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.and
// CHECK:     yul.or
// CHECK:     yul.xor
// CHECK:     yul.not
// CHECK:     yul.shl
// CHECK:     yul.shr
// CHECK:     yul.sar
// CHECK:     yul.byte
// CHECK:     yul.clz

// CHECK: sol.func @{{.*comparison.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.cmp ult
// CHECK:     yul.cmp ugt
// CHECK:     yul.cmp slt
// CHECK:     yul.cmp sgt
// CHECK:     yul.cmp eq
// The `iszero` builtin has no op of its own: it compares against zero.
// CHECK:     %[[ZERO:.*]] = yul.constant 0
// CHECK:     yul.cmp eq, %{{.*}}, %[[ZERO]]

// CHECK: sol.func @{{.*memory.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.mload
// CHECK:     yul.mstore
// CHECK:     yul.mstore8
// CHECK:     yul.mcopy
// CHECK:     yul.msize
// CHECK:     yul.keccak256

// CHECK: sol.func @{{.*storage.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.sload
// CHECK:     yul.sstore
// CHECK:     yul.tload
// CHECK:     yul.tstore

// CHECK: sol.func @{{.*calls.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.call
// CHECK:     yul.callcode
// CHECK:     yul.static_call
// CHECK:     yul.delegate_call
// CHECK:     yul.create
// CHECK:     yul.create2

// CHECK: sol.func @{{.*context.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.address
// CHECK:     yul.balance
// CHECK:     yul.selfbalance
// CHECK:     yul.caller
// CHECK:     yul.callvalue
// CHECK:     yul.gas
// CHECK:     yul.gasprice
// CHECK:     yul.gaslimit
// CHECK:     yul.origin
// CHECK:     yul.chainid
// CHECK:     yul.basefee
// CHECK:     yul.blobbasefee
// CHECK:     yul.coinbase
// CHECK:     yul.timestamp
// CHECK:     yul.number
// CHECK:     yul.prevrandao
// CHECK:     yul.blockhash
// CHECK:     yul.blobhash

// CHECK: sol.func @{{.*data.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.calldataload
// CHECK:     yul.calldatasize
// CHECK:     yul.calldatacopy
// CHECK:     yul.returndatasize
// CHECK:     yul.returndatacopy
// CHECK:     yul.codesize
// CHECK:     yul.codecopy
// CHECK:     yul.extcodesize
// CHECK:     yul.extcodehash
// CHECK:     yul.extcodecopy

// CHECK: sol.func @{{.*logs.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.log %{{.*}}, %{{.*}}{{$}}
// CHECK:     yul.log %{{.*}}, %{{.*}} topics(%{{.*}})
// CHECK:     yul.log %{{.*}}, %{{.*}} topics(%{{.*}}, %{{.*}})
// CHECK:     yul.log %{{.*}}, %{{.*}} topics(%{{.*}}, %{{.*}}, %{{.*}})
// CHECK:     yul.log %{{.*}}, %{{.*}} topics(%{{.*}}, %{{.*}}, %{{.*}}, %{{.*}})

// CHECK: sol.func @{{.*halting.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.selfdestruct
// CHECK:     yul.invalid
// CHECK:     yul.stop
// CHECK:     yul.revert
// CHECK:     yul.return

// CHECK: sol.func @{{.*memory_safe.*}}
// CHECK:   sol.inline_asm attributes {memory_safe} {

contract C {
    uint256 slot0;
    uint256 transient slot1;

    function arithmetic(uint256 x, uint256 y) public pure returns (uint256 r) {
        assembly {
            let a := add(x, 1)
            r := mul(a, y)
            r := sub(r, y)
            r := div(r, y)
            r := sdiv(r, y)
            r := mod(r, y)
            r := smod(r, y)
            r := exp(r, y)
            r := addmod(r, y, a)
            r := mulmod(r, y, a)
            r := signextend(1, r)
        }
    }

    function bitwise(uint256 x, uint256 y) public pure returns (uint256 r) {
        assembly {
            r := and(x, y)
            r := or(r, y)
            r := xor(r, y)
            r := not(r)
            r := shl(1, r)
            r := shr(1, r)
            r := sar(1, r)
            r := byte(0, r)
            r := clz(r)
        }
    }

    function comparison(uint256 x, uint256 y) public pure returns (uint256 r) {
        assembly {
            r := lt(x, y)
            r := gt(x, y)
            r := slt(x, y)
            r := sgt(x, y)
            r := eq(x, y)
            r := iszero(x)
        }
    }

    function memory_ops(uint256 x) public pure returns (uint256 r) {
        assembly {
            r := mload(x)
            mstore(x, r)
            mstore8(x, r)
            mcopy(x, x, 32)
            r := msize()
            r := keccak256(x, 32)
        }
    }

    function storage_ops(uint256 x) public returns (uint256 r) {
        assembly {
            r := sload(slot0.slot)
            sstore(slot0.slot, x)
            r := tload(slot1.slot)
            tstore(slot1.slot, x)
        }
    }

    function calls(uint256 x) public returns (uint256 r) {
        assembly {
            r := call(x, x, x, x, x, x, x)
            r := callcode(x, x, x, x, x, x, x)
            r := staticcall(x, x, x, x, x, x)
            r := delegatecall(x, x, x, x, x, x)
            r := create(x, x, x)
            r := create2(x, x, x, x)
        }
    }

    function context(uint256 x) public view returns (uint256 r) {
        assembly {
            r := address()
            r := balance(x)
            r := selfbalance()
            r := caller()
            r := callvalue()
            r := gas()
            r := gasprice()
            r := gaslimit()
            r := origin()
            r := chainid()
            r := basefee()
            r := blobbasefee()
            r := coinbase()
            r := timestamp()
            r := number()
            r := prevrandao()
            r := blockhash(x)
            r := blobhash(x)
        }
    }

    function data(uint256 x) public view returns (uint256 r) {
        assembly {
            r := calldataload(x)
            r := calldatasize()
            calldatacopy(x, x, x)
            r := returndatasize()
            returndatacopy(x, x, x)
            r := codesize()
            codecopy(x, x, x)
            r := extcodesize(x)
            r := extcodehash(x)
            extcodecopy(x, x, x, x)
        }
    }

    function logs(uint256 x) public {
        assembly {
            log0(0, 32)
            log1(0, 32, x)
            log2(0, 32, x, x)
            log3(0, 32, x, x, x)
            log4(0, 32, x, x, x, x)
        }
    }

    function halting(uint256 x) public {
        assembly {
            if eq(x, 1) { selfdestruct(x) }
            if eq(x, 2) { invalid() }
            if eq(x, 3) { stop() }
            if eq(x, 4) { revert(0, 0) }
            return(0, 32)
        }
    }

    function memory_safe() public pure returns (uint256 r) {
        assembly ("memory-safe") {
            r := mload(0x40)
        }
    }
}
