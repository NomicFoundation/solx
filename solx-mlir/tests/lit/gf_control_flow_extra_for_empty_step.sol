// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A `for` loop whose step clause is empty: both backends emit only the cond
// and body regions (no `step` region) and increment the counter in the body.

// CHECK: sol.func @{{.*no_step.*}}
// CHECK:   sol.for cond {
// CHECK:     sol.condition %{{.*}}
// CHECK:   } body {
// CHECK:     sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:     sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:     sol.yield
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function no_step(uint256 n) public pure returns (uint256) {
        uint256 s = 0;
        for (uint256 i = 0; i < n;) {
            s = s + i;
            i = i + 1;
        }
        return s;
    }
}
