// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*pushAssign.*}}
// CHECK: %[[ARR:.*]] = sol.addr_of @{{.*arr.*}} : !sol.array<? x ui256, Storage>
// CHECK: %[[SLOT:.*]] = sol.push %[[ARR]] : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK: %[[V:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.store %[[V]], %[[SLOT]] : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256[] arr;
    function pushAssign(uint256 v) public { arr.push() = v; }
}
