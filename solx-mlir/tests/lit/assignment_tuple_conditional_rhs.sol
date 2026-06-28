// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*cond.*}}
// CHECK: sol.if %{{[0-9]+}}
// CHECK: %[[L0:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui8, Stack>, ui8
// CHECK: %[[L1:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui8, Stack>, ui8
// CHECK-DAG: %[[C0:.*]] = sol.cast %[[L0]] : ui8 to ui256
// CHECK-DAG: %[[C1:.*]] = sol.cast %[[L1]] : ui8 to ui256
// CHECK-DAG: sol.store %[[C0]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>
// CHECK-DAG: sol.store %[[C1]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>

contract C {
    function cond(bool f) public pure returns (uint256, uint256) {
        uint256 a; uint256 b;
        (a, b) = f ? (1, 2) : (3, 4);
        return (a, b);
    }
}
