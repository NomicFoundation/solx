// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*s.*}}() -> (ui256, !sol.string<Memory>) attributes {{.*}}selector = -2034821918 : i32
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*s.*}} : !sol.struct<(ui256, !sol.string<Storage>), Storage>
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[BASE]], %[[I0]] : !sol.struct<(ui256, !sol.string<Storage>), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK:   %[[P1:.*]] = sol.gep %[[BASE]], %[[I1]] : !sol.struct<(ui256, !sol.string<Storage>), Storage>, ui64, !sol.string<Storage>
// CHECK:   %[[V1:.*]] = sol.data_loc_cast %[[P1]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.return %[[V0]], %[[V1]] : ui256, !sol.string<Memory>

contract C {
    struct S { uint256 a; string label; }

    S public s;
}
