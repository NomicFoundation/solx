// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// push/pop on the inner array of a storage uint256[][]. grid[i] is a sol.gep
// into the outer storage array that yields the inner !sol.array<? x ui256,
// Storage> value directly (storage geps don't insert an intermediate load).
// .push then emits sol.push -> a ptr to the new slot, followed by a store of the
// value; .pop emits sol.pop. Both backends produce identical chains; emission
// order differs (solx alphabetical: popInner, pushInner; solc source order:
// pushInner, popInner), hence split prefixes.

// CHECK-SOLX: sol.state_var @{{.*grid.*}} slot 0 offset 0 : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK-SOLX: sol.func {{.*}}popInner
// CHECK-SOLX:   sol.addr_of @{{.*grid.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK-SOLX:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>, ui256, !sol.array<? x ui256, Storage>
// CHECK-SOLX:   sol.pop %{{.*}} : !sol.array<? x ui256, Storage>
// CHECK-SOLX: sol.func {{.*}}pushInner
// CHECK-SOLX:   sol.addr_of @{{.*grid.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK-SOLX:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>, ui256, !sol.array<? x ui256, Storage>
// CHECK-SOLX:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK-SOLX:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK-SOLC: sol.state_var @{{.*grid.*}} slot 0 offset 0 : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK-SOLC: sol.func {{.*}}pushInner
// CHECK-SOLC:   sol.addr_of @{{.*grid.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK-SOLC:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>, ui256, !sol.array<? x ui256, Storage>
// CHECK-SOLC:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK-SOLC:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK-SOLC: sol.func {{.*}}popInner
// CHECK-SOLC:   sol.addr_of @{{.*grid.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>
// CHECK-SOLC:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage>, ui256, !sol.array<? x ui256, Storage>
// CHECK-SOLC:   sol.pop %{{.*}} : !sol.array<? x ui256, Storage>

contract C {
    uint256[][] grid;

    function pushInner(uint256 i, uint256 v) public {
        grid[i].push(v);
    }

    function popInner(uint256 i) public {
        grid[i].pop();
    }
}
