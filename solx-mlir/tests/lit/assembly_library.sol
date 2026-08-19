// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// An internal library function carries its assembly into the calling contract's module.
// CHECK: sol.func @{{.*twice.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.constant 2
// CHECK:     yul.mul

library L {
    function twice(uint256 x) internal pure returns (uint256 r) {
        assembly {
            r := mul(x, 2)
        }
    }
}

contract C {
    function use(uint256 x) public pure returns (uint256 r) {
        r = L.twice(x);
    }
}
