// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Auto-generated public getter for a dynamic array state var: the synthesized
// getter takes an index argument. Both backends agree on the signature, selector,
// and the addr_of/gep/load/return shape. They DIVERGE on the bounds check:
//   - solx emits an EXPLICIT bounds check (sol.length -> sol.cmp lt -> sol.require)
//     and then a plain sol.gep.
//   - solc skips the explicit check and instead emits `sol.gep ... no_panic_bounds`.
// Split prefixes pin each backend's real shape (op-level divergence, see report).

// CHECK-SOLX: sol.func @{{.*items.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -1078840878 : i32
// CHECK-SOLX:   %[[A:.*]] = sol.addr_of @{{.*items.*}} : !sol.array<? x ui256, Storage>
// CHECK-SOLX:   %[[LEN:.*]] = sol.length %[[A]] : !sol.array<? x ui256, Storage>
// CHECK-SOLX:   %[[OK:.*]] = sol.cmp lt, %arg0, %[[LEN]] : ui256
// CHECK-SOLX:   sol.require %[[OK]]()
// CHECK-SOLX:   %[[P:.*]] = sol.gep %[[A]], %arg0 : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK-SOLX:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-SOLX:   sol.return %[[V]] : ui256

// CHECK-SOLC: sol.func @{{.*items.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -1078840878 : i32
// CHECK-SOLC:   %[[A:.*]] = sol.addr_of @{{.*items.*}} : !sol.array<? x ui256, Storage>
// CHECK-SOLC:   %[[P:.*]] = sol.gep %[[A]], %arg0 no_panic_bounds : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK-SOLC:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-SOLC:   sol.return %[[V]] : ui256

contract C {
    uint256[] public items;
}
