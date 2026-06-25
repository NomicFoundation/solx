// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Chained assignment `a = b = c`: the inner assignment yields a value that the
// outer assignment also stores, so `c` flows into both `b` and `a`. solx threads
// the loaded `c` directly into both stores; solc stores into `b` then reloads `b`
// for the store into `a` (an extra load). The shared, backend-agnostic shape is:
// load `c`, store into both locals, then load both for the return tuple. The two
// destination stores are CHECK-DAG (order is unobservable for distinct locals).

// CHECK: sol.func @{{.*f.*}}(%arg0: ui256) -> (ui256, ui256)
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.return %{{.*}}, %{{.*}} : ui256, ui256

contract C {
    function f(uint256 c) public pure returns (uint256, uint256) {
        uint256 a;
        uint256 b;
        a = b = c;
        return (a, b);
    }
}
