// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `arr.push(s)` where the element type is a reference (a struct) appends by
// allocating the new storage slot (`sol.push`) and copying the source memory
// aggregate into it via a memory->storage `sol.copy` (rather than a scalar
// store). Both backends emit the same push-then-copy sequence.

// CHECK: %[[A:.*]] = sol.addr_of @{{.*}} : !sol.array<? x !sol.struct<(ui256, ui256), Storage>, Storage>
// CHECK: %[[SLOT:.*]] = sol.push %[[A]] : !sol.array<? x !sol.struct<(ui256, ui256), Storage>, Storage> -> !sol.struct<(ui256, ui256), Storage>
// CHECK: %[[S:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.struct<(ui256, ui256), Memory>, Stack>, !sol.struct<(ui256, ui256), Memory>
// CHECK: sol.copy %[[S]], %[[SLOT]] : !sol.struct<(ui256, ui256), Memory>, !sol.struct<(ui256, ui256), Storage>

contract C {
    struct S { uint256 a; uint256 b; }
    S[] arr;

    function f(S memory s) external {
        arr.push(s);
    }
}
