// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `arr.push() = v` — the no-argument `push` appends a default element and returns
// a reference to the freshly-created slot; that reference is the assignment
// lvalue. The emitter materialises the new slot via `sol.push` and stores the
// RHS into it. solc's nascent MLIR backend rejects this form (NYI), so this is a
// solx-only check.

// CHECK: sol.func @{{.*pushAssign.*}}
// CHECK: %[[ARR:.*]] = sol.addr_of @{{arr.*}} : !sol.array<? x ui256, Storage>
// CHECK: %[[SLOT:.*]] = sol.push %[[ARR]] : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK: %[[V:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.store %[[V]], %[[SLOT]] : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256[] arr;
    function pushAssign(uint256 v) public { arr.push() = v; }
}
