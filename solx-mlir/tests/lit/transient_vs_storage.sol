// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*persistent.*}} slot 0 offset 0 : ui256
// CHECK: sol.state_var @{{.*guard.*}} transient slot 0 offset 0 : ui256

// CHECK: sol.func @{{.*bump.*}}
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Transient>, ui256
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.cadd
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Transient>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 persistent;
    uint256 transient guard;

    function bump() public {
        guard = guard + 1;
        persistent = guard;
    }
}
