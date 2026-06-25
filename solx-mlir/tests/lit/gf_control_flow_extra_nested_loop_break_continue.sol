// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// break/continue nested inside two `for` loops bind to the innermost loop.
// Both backends emit nested sol.for regions with the continue/break inside
// the inner body's sol.if.

// CHECK: sol.func @{{.*nested.*}}
// CHECK:   sol.for cond {
// CHECK:     sol.condition %{{.*}}
// CHECK:   } body {
// CHECK:     sol.for cond {
// CHECK:       sol.condition %{{.*}}
// CHECK:     } body {
// CHECK:       sol.if %{{.*}} {
// CHECK:         sol.continue
// CHECK:       sol.if %{{.*}} {
// CHECK:         sol.break
// CHECK:     } step {
// CHECK:       sol.yield
// CHECK:   } step {
// CHECK:     sol.yield
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function nested(uint256 n) public pure returns (uint256) {
        uint256 s = 0;
        for (uint256 i = 0; i < n; i++) {
            for (uint256 j = 0; j < n; j++) {
                if (j == 2) { continue; }
                if (j == 5) { break; }
                s = s + j;
            }
        }
        return s;
    }
}
