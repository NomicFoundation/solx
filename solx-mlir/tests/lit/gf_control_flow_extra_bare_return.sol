// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A bare `return;` (no value) inside a void function: an early exit guarded by
// an `if`, followed by an implicit trailing `sol.return`.

// CHECK: sol.func @{{.*bare_return.*}}
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.return
// CHECK:   sol.addr_of @{{.*x.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.return

contract C {
    uint256 public x;
    function bare_return(uint256 a) public {
        if (a == 0) {
            return;
        }
        x = a;
    }
}
