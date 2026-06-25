// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A transient var coexists with a regular storage var. They live in distinct
// address spaces (Transient vs Storage) with independent slot numbering: both
// land at `slot 0 offset 0`. A read-modify-write of the transient var and a
// copy into storage keep the address spaces straight. solx and solc agree on
// the op set (ordering differs, so the body uses CHECK-DAG).

// CHECK-DAG: sol.state_var @{{.*persistent.*}} slot 0 offset 0 : ui256
// CHECK-DAG: sol.state_var @{{.*guard.*}} transient slot 0 offset 0 : ui256

// CHECK: sol.func @{{.*bump.*}}
// CHECK-DAG:   sol.addr_of @{{.*guard.*}} : !sol.ptr<ui256, Transient>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Transient>, ui256
// CHECK-DAG:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Transient>
// CHECK-DAG:   sol.addr_of @{{.*persistent.*}} : !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 persistent;
    uint256 transient guard;

    function bump() public {
        guard = guard + 1;
        persistent = guard;
    }
}
