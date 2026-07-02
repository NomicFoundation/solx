// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.new
// CHECK: sol.try
// CHECK: fallback {

contract D {
    uint256 x;

    constructor(uint256 a, uint256 b) {
        x = a + b;
    }
}

contract C {
    function f() external returns (address) {
        try new D({b: 2, a: 1}) returns (D d) {
            return address(d);
        } catch {
            return address(0);
        }
    }
}
