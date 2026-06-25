// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Multiple early-return paths: each guarded `return` lives inside its own
// sol.if, followed by the fall-through final return. Both backends agree.

// CHECK: sol.func @{{.*early.*}}
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.return %{{.*}} : ui256
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.return %{{.*}} : ui256
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.return %{{.*}} : ui256
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function early(uint256 a) public pure returns (uint256) {
        if (a == 0) { return 100; }
        if (a == 1) { return 200; }
        if (a == 2) { return 300; }
        return 999;
    }
}
