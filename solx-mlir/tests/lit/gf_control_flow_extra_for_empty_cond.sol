// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A `for` loop with an empty condition clause: both backends synthesize a
// `sol.constant true` condition. The early `return` inside the body breaks out.

// CHECK: sol.func @{{.*no_cond.*}}
// CHECK:   sol.for cond {
// CHECK:     %[[T:.*]] = sol.constant true
// CHECK:     sol.condition %[[T]]
// CHECK:   } body {
// CHECK:     sol.if %{{.*}} {
// CHECK:       sol.return %{{.*}} : ui256
// CHECK:   } step {
// CHECK:     sol.yield

contract C {
    function no_cond(uint256 n) public pure returns (uint256) {
        uint256 s = 0;
        for (uint256 i = 0; ; i++) {
            if (i >= n) { return s; }
            s = s + i;
        }
    }
}
