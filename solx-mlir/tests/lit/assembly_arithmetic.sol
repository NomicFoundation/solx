// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*arithmetic.*}}
// CHECK: yul.add
// CHECK: yul.mul
// CHECK: yul.sub
// CHECK: yul.div
// CHECK: yul.mod
// CHECK: yul.exp

// CHECK: sol.func @{{.*signed.*}}
// CHECK: yul.sdiv
// CHECK: yul.smod
// CHECK: yul.signextend
// CHECK: yul.addmod
// CHECK: yul.mulmod

contract C {
    function arithmetic(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := add(a, b)
            r := mul(r, b)
            r := sub(r, a)
            r := div(r, b)
            r := mod(r, a)
            r := exp(a, b)
        }
    }

    function signed(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := sdiv(a, b)
            r := smod(a, b)
            r := signextend(a, r)
            r := addmod(a, b, r)
            r := mulmod(a, b, r)
        }
    }
}
