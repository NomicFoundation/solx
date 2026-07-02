// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*items.*}}(%arg0: ui256) -> (ui256, i1) attributes {{.*}}selector = -1078840878 : i32
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*items.*}} : !sol.array<? x !sol.struct<(ui256, i1), Storage>, Storage>
// CHECK:   %[[E:.*]] = sol.gep %[[A]], %arg0 no_panic_bounds : !sol.array<? x !sol.struct<(ui256, i1), Storage>, Storage>, ui256, !sol.struct<(ui256, i1), Storage>
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[E]], %[[I0]] : !sol.struct<(ui256, i1), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK:   %[[P1:.*]] = sol.gep %[[E]], %[[I1]] : !sol.struct<(ui256, i1), Storage>, ui64, !sol.ptr<i1, Storage>
// CHECK:   %[[V1:.*]] = sol.load %[[P1]] : !sol.ptr<i1, Storage>, i1
// CHECK:   sol.return %[[V0]], %[[V1]] : ui256, i1

contract C {
    struct Item { uint256 id; bool ok; }

    Item[] public items;
}
