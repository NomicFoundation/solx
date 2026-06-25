// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A `for` loop whose init clause is empty (the counter is declared before the
// loop). Both backends emit cond/body/step regions identically.

// CHECK: sol.func @{{.*no_init.*}}
// CHECK:   sol.for cond {
// CHECK:     sol.condition %{{.*}}
// CHECK:   } body {
// CHECK:     sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:     sol.yield
// CHECK:   } step {
// CHECK:     sol.add %{{.*}}, %{{.*}} : ui256
// CHECK:     sol.yield
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function no_init(uint256 n) public pure returns (uint256) {
        uint256 i = 0;
        uint256 s = 0;
        for (; i < n; i++) {
            s = s + i;
        }
        return s;
    }
}
