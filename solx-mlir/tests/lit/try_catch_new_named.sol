// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[A:.*]] = sol.cast %c1_ui8
// CHECK: %[[B:.*]] = sol.cast %c2_ui8
// CHECK: sol.new "D{{.*}}ctor(%[[A]], %[[B]] {{.*}}try
// CHECK: sol.try
// CHECK: fallback {

contract C {
    function f() external returns (uint256) {
        try new D({b: 2, a: 1}) returns (D) {
            return 0;
        } catch {
            return 1;
        }
    }
}

contract D {
    uint256 x;

    constructor(uint256 a, uint256 b) {
        x = a + b;
    }
}
