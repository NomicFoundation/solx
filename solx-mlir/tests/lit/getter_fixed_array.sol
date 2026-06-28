// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*fixed_items.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -2078704799 : i32
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*fixed_items.*}} : !sol.array<3 x ui256, Storage>
// CHECK:   %[[P:.*]] = sol.gep %[[A]], %arg0 no_panic_bounds : !sol.array<3 x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

contract C {
    uint256[3] public fixed_items;
}
