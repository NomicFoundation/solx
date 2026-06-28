// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.state_var @{{.*origin.*}} slot 0 offset 0 : !sol.struct<(ui256, ui256), Storage>

// CHECK: sol.func @{{.*origin.*}}() -> (ui256, ui256) attributes {{.*}}selector = -1819582670 : i32
// CHECK:   %[[S:.*]] = sol.addr_of @{{.*origin.*}} : !sol.struct<(ui256, ui256), Storage>
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[S]], %[[I0]] : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK:   %[[P1:.*]] = sol.gep %[[S]], %[[I1]] : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V1:.*]] = sol.load %[[P1]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V0]], %[[V1]] : ui256, ui256

contract C {
    struct Point { uint256 x; uint256 y; }
    Point public origin;
}
