// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}} attributes {{{.*}}kind = #Constructor
// CHECK:   sol.inline_asm {
// CHECK:     yul.sstore %{{.*}}, %c9_i256

// CHECK: sol.func @{{.*arithmetic.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[X:.*]] = yul.load %{{.*}} : !yul.ptr -> i256
// CHECK:     %[[SUM:.*]] = yul.add %[[X]], %c3_i256
// CHECK:     %[[V:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[SUM]], %[[V]] : i256, !yul.ptr
// CHECK:     yul.mul %{{.*}}, %c5_i256
// CHECK:     yul.sub %{{.*}}, %c7_i256
// CHECK:     yul.div %{{.*}}, %c11_i256
// CHECK:     yul.sdiv %{{.*}}, %c13_i256
// CHECK:     yul.mod %{{.*}}, %c17_i256
// CHECK:     yul.smod %{{.*}}, %c19_i256
// CHECK:     yul.exp %{{.*}}, %c23_i256
// CHECK:     yul.addmod %{{.*}}, %c29_i256, %c31_i256
// CHECK:     yul.mulmod %{{.*}}, %c37_i256, %c41_i256
// CHECK:     yul.signextend %c2_i256, %{{.*}}

// CHECK: sol.func @{{.*bitwise.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.and %{{.*}}, %c3_i256
// CHECK:     yul.or %{{.*}}, %c5_i256
// CHECK:     yul.xor %{{.*}}, %c7_i256
// CHECK:     yul.not %{{.*}}
// CHECK:     yul.shl %c11_i256, %{{.*}}
// CHECK:     yul.shr %c13_i256, %{{.*}}
// CHECK:     yul.sar %c17_i256, %{{.*}}
// CHECK:     yul.byte %c19_i256, %{{.*}}
// CHECK:     yul.clz %{{.*}}

// CHECK: sol.func @{{.*comparison.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.cmp ult, %c1_i256, %c2_i256
// CHECK:     yul.cmp ugt, %c3_i256, %c5_i256
// CHECK:     yul.cmp slt, %c7_i256, %c11_i256
// CHECK:     yul.cmp sgt, %c13_i256, %c17_i256
// CHECK:     yul.cmp eq, %c19_i256, %c23_i256
// CHECK:     %[[ZERO:.*]] = yul.constant 0
// CHECK:     yul.cmp eq, %c29_i256, %[[ZERO]]

// CHECK: sol.func @{{.*memory_ops.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.mload %c1_i256
// CHECK:     yul.mstore %c11_i256, %c12_i256
// CHECK:     yul.mstore8 %c21_i256, %c22_i256
// CHECK:     yul.mcopy %c31_i256, %c32_i256, %c33_i256
// CHECK:     yul.msize
// CHECK:     yul.keccak256 %c41_i256, %c42_i256

// CHECK: sol.func @{{.*storage_ops.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.sload
// CHECK:     yul.sstore %{{.*}}, %c7_i256
// CHECK:     yul.tload
// CHECK:     yul.tstore %{{.*}}, %c11_i256

// CHECK: sol.func @{{.*calls.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.call %c1_i256, %c2_i256, %c3_i256, %c4_i256, %c5_i256, %c6_i256, %c7_i256
// CHECK:     yul.callcode %c11_i256, %c12_i256, %c13_i256, %c14_i256, %c15_i256, %c16_i256, %c17_i256
// CHECK:     yul.static_call %c21_i256, %c22_i256, %c23_i256, %c24_i256, %c25_i256, %c26_i256
// CHECK:     yul.delegate_call %c31_i256, %c32_i256, %c33_i256, %c34_i256, %c35_i256, %c36_i256
// CHECK:     yul.create %c41_i256, %c42_i256, %c43_i256
// CHECK:     yul.create2 %c51_i256, %c52_i256, %c53_i256, %c54_i256

// CHECK: sol.func @{{.*context.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.address
// CHECK:     yul.balance %c1_i256
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
// CHECK:     yul.blockhash %c2_i256
// CHECK:     yul.blobhash %c3_i256

// CHECK: sol.func @{{.*data_ops.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.calldataload %c1_i256
// CHECK:     yul.calldatasize
// CHECK:     yul.calldatacopy %c11_i256, %c12_i256, %c13_i256
// CHECK:     yul.returndatasize
// CHECK:     yul.returndatacopy %c21_i256, %c22_i256, %c23_i256
// CHECK:     yul.codesize
// CHECK:     yul.codecopy %c31_i256, %c32_i256, %c33_i256
// CHECK:     yul.extcodesize %c41_i256
// CHECK:     yul.extcodehash %c43_i256
// CHECK:     yul.extcodecopy %c51_i256, %c52_i256, %c53_i256, %c54_i256

// CHECK: sol.func @{{.*logs.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.log %c1_i256, %c2_i256{{$}}
// CHECK:     yul.log %c11_i256, %c12_i256 topics(%c13_i256)
// CHECK:     yul.log %c21_i256, %c22_i256 topics(%c23_i256, %c24_i256)
// CHECK:     yul.log %c31_i256, %c32_i256 topics(%c33_i256, %c34_i256, %c35_i256)
// CHECK:     yul.log %c41_i256, %c42_i256 topics(%c43_i256, %c44_i256, %c45_i256, %c46_i256)

// CHECK: sol.func @{{.*halting.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.selfdestruct %c101_i256
// CHECK:     yul.invalid
// CHECK:     yul.stop
// CHECK:     yul.revert %c102_i256, %c103_i256
// CHECK:     yul.return %c104_i256, %c105_i256

// CHECK: sol.func @{{.*memory_safe.*}}
// CHECK:   sol.inline_asm attributes {memory_safe} {

contract C {
    uint256 slot0;
    uint256 transient slot1;

    constructor() {
        assembly {
            sstore(slot0.slot, 9)
        }
    }

    function arithmetic(uint256 x) public pure returns (uint256 r) {
        assembly {
            let v := add(x, 3)
            v := mul(v, 5)
            v := sub(v, 7)
            v := div(v, 11)
            v := sdiv(v, 13)
            v := mod(v, 17)
            v := smod(v, 19)
            v := exp(v, 23)
            v := addmod(v, 29, 31)
            v := mulmod(v, 37, 41)
            r := signextend(2, v)
        }
    }

    function bitwise(uint256 x) public pure returns (uint256 r) {
        assembly {
            let v := and(x, 3)
            v := or(v, 5)
            v := xor(v, 7)
            v := not(v)
            v := shl(11, v)
            v := shr(13, v)
            v := sar(17, v)
            v := byte(19, v)
            r := clz(v)
        }
    }

    function comparison() public pure returns (uint256 r) {
        assembly {
            r := lt(1, 2)
            r := gt(3, 5)
            r := slt(7, 11)
            r := sgt(13, 17)
            r := eq(19, 23)
            r := iszero(29)
        }
    }

    function memory_ops() public pure returns (uint256 r) {
        assembly {
            r := mload(1)
            mstore(11, 12)
            mstore8(21, 22)
            mcopy(31, 32, 33)
            r := msize()
            r := keccak256(41, 42)
        }
    }

    function storage_ops() public returns (uint256 r) {
        assembly {
            r := sload(slot0.slot)
            sstore(slot0.slot, 7)
            r := tload(slot1.slot)
            tstore(slot1.slot, 11)
        }
    }

    function calls() public returns (uint256 r) {
        assembly {
            r := call(1, 2, 3, 4, 5, 6, 7)
            r := callcode(11, 12, 13, 14, 15, 16, 17)
            r := staticcall(21, 22, 23, 24, 25, 26)
            r := delegatecall(31, 32, 33, 34, 35, 36)
            r := create(41, 42, 43)
            r := create2(51, 52, 53, 54)
        }
    }

    function context() public view returns (uint256 r) {
        assembly {
            r := address()
            r := balance(1)
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
            r := blockhash(2)
            r := blobhash(3)
        }
    }

    function data_ops() public view returns (uint256 r) {
        assembly {
            r := calldataload(1)
            r := calldatasize()
            calldatacopy(11, 12, 13)
            r := returndatasize()
            returndatacopy(21, 22, 23)
            r := codesize()
            codecopy(31, 32, 33)
            r := extcodesize(41)
            r := extcodehash(43)
            extcodecopy(51, 52, 53, 54)
        }
    }

    function logs() public {
        assembly {
            log0(1, 2)
            log1(11, 12, 13)
            log2(21, 22, 23, 24)
            log3(31, 32, 33, 34, 35)
            log4(41, 42, 43, 44, 45, 46)
        }
    }

    function halting(uint256 x) public {
        assembly {
            if eq(x, 1) { selfdestruct(101) }
            if eq(x, 2) { invalid() }
            if eq(x, 3) { stop() }
            if eq(x, 4) { revert(102, 103) }
            return(104, 105)
        }
    }

    function memory_safe() public pure returns (uint256 r) {
        assembly ("memory-safe") {
            r := mload(0x40)
        }
    }
}
