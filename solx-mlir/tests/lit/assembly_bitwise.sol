// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*bits.*}}
// CHECK: yul.and
// CHECK: yul.or
// CHECK: yul.xor
// CHECK: yul.not
// CHECK: yul.shl
// CHECK: yul.shr
// CHECK: yul.sar
// CHECK: yul.byte

contract C {
    function bits(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := and(a, b)
            r := or(r, a)
            r := xor(r, b)
            r := not(r)
            r := shl(a, r)
            r := shr(b, r)
            r := sar(a, r)
            r := byte(a, r)
        }
    }
}
