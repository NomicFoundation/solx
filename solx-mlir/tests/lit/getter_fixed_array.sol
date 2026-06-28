// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Auto-generated getter for a fixed-size array state var. Both backends agree on
// the signature, selector, and addr_of/gep/load/return shape, but DIVERGE on the
// index bounds check (same split as getter_array.sol, here the length is the
// compile-time constant size):
//   - solx emits an EXPLICIT check (sol.constant size -> sol.cmp lt -> sol.require)
//     then a plain sol.gep.
//   - solc skips it and emits `sol.gep ... no_panic_bounds`.

// CHECK-SOLX: sol.func @{{.*fixed_items.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -2078704799 : i32
// CHECK-SOLX:   %[[A:.*]] = sol.addr_of @{{.*fixed_items.*}} : !sol.array<3 x ui256, Storage>
// CHECK-SOLX:   %[[N:.*]] = sol.constant 3 : ui256
// CHECK-SOLX:   %[[OK:.*]] = sol.cmp lt, %arg0, %[[N]] : ui256
// CHECK-SOLX:   sol.require %[[OK]]()
// CHECK-SOLX:   %[[P:.*]] = sol.gep %[[A]], %arg0 : !sol.array<3 x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK-SOLX:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-SOLX:   sol.return %[[V]] : ui256

// CHECK-SOLC: sol.func @{{.*fixed_items.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -2078704799 : i32
// CHECK-SOLC:   %[[A:.*]] = sol.addr_of @{{.*fixed_items.*}} : !sol.array<3 x ui256, Storage>
// CHECK-SOLC:   %[[P:.*]] = sol.gep %[[A]], %arg0 no_panic_bounds : !sol.array<3 x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK-SOLC:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-SOLC:   sol.return %[[V]] : ui256

contract C {
    uint256[3] public fixed_items;
}
