// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A do-while loop with both `continue` and `break` in its body. Both backends
// emit a `sol.do { ... } while { sol.condition }` with the jumps inside ifs.

// CHECK: sol.func @{{.*dw.*}}
// CHECK:   sol.do {
// CHECK:     sol.if %{{.*}} {
// CHECK:       sol.continue
// CHECK:     sol.if %{{.*}} {
// CHECK:       sol.break
// CHECK:     sol.yield
// CHECK:   } while {
// CHECK:     sol.condition %{{.*}}
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function dw(uint256 n) public pure returns (uint256) {
        uint256 i = 0;
        uint256 s = 0;
        do {
            i = i + 1;
            if (i == 2) { continue; }
            if (i == 7) { break; }
            s = s + i;
        } while (i < n);
        return s;
    }
}
