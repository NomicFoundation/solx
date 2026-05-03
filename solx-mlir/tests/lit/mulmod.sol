// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.mulmod {{.*}} : ui256

contract C {
    function f(uint256 x, uint256 y, uint256 m) public pure returns (uint256) {
        return mulmod(x, y, m);
    }
}
