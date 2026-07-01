// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*grid.*}} slot 0 offset 0 : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK: sol.func {{.*}}popInner
// CHECK:   sol.addr_of @{{.*grid.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>, ui256, !sol.array<? x ui256, Storage>
// CHECK:   sol.pop %{{.*}} : !sol.array<? x ui256, Storage>
// CHECK: sol.func {{.*}}pushInner
// CHECK:   sol.addr_of @{{.*grid.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>, ui256, !sol.array<? x ui256, Storage>
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256[][] grid;

    function popInner(uint256 i) public {
        grid[i].pop();
    }

    function pushInner(uint256 i, uint256 v) public {
        grid[i].push(v);
    }
}
