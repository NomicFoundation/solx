// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `continue` and `break` inside a `for` loop body. `continue` jumps to the step
// region, `break` exits the loop. Both backends emit the jumps inside ifs in
// the body region.

// CHECK: sol.func @{{.*fc.*}}
// CHECK:   sol.for cond {
// CHECK:     sol.condition %{{.*}}
// CHECK:   } body {
// CHECK:     sol.if %{{.*}} {
// CHECK:       sol.continue
// CHECK:     sol.if %{{.*}} {
// CHECK:       sol.break
// CHECK:     sol.yield
// CHECK:   } step {
// CHECK:     sol.yield

contract C {
    function fc(uint256 n) public pure returns (uint256) {
        uint256 s = 0;
        for (uint256 i = 0; i < n; i++) {
            if (i == 3) { continue; }
            if (i == 8) { break; }
            s = s + i;
        }
        return s;
    }
}
