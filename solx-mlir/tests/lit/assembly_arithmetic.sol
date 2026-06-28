// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Inline-assembly arithmetic and bitwise opcodes lower to Yul-dialect ops
// (rule 16: Yul never crosses into the Sol dialect).

// CHECK: sol.func @{{.*arith.*}}
// CHECK: yul.add
// CHECK: yul.mul
// CHECK: yul.sub
// CHECK: yul.div
// CHECK: yul.mod
// CHECK: yul.exp

// CHECK: sol.func @{{.*bits.*}}
// CHECK: yul.and
// CHECK: yul.or
// CHECK: yul.xor
// CHECK: yul.not
// CHECK: yul.shl
// CHECK: yul.shr

// CHECK: sol.func @{{.*signed.*}}
// CHECK: yul.sdiv
// CHECK: yul.smod
// CHECK: yul.sar
// CHECK: yul.byte
// CHECK: yul.signextend
// CHECK: yul.addmod
// CHECK: yul.mulmod

contract C {
    function arith(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := add(a, b)
            r := mul(r, b)
            r := sub(r, a)
            r := div(r, b)
            r := mod(r, a)
            r := exp(a, b)
        }
    }

    function bits(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := and(a, b) // EXPLAIN: how are these arithmetic?
            r := or(r, a)
            r := xor(r, b)
            r := not(r)
            r := shl(a, r)
            r := shr(b, r)
        }
    }

    function signed(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := sdiv(a, b)
            r := smod(a, b)
            r := sar(a, r) // EXPLAIN: are these last 5 still arithmetic
            r := byte(a, r)
            r := signextend(a, r)
            r := addmod(a, b, r)
            r := mulmod(a, b, r)
        }
    }
}
