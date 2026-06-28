// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Auto-generated getter for a dynamic array OF structs: an index argument selects
// the element, then the struct's value-type fields are returned as a tuple. The
// struct-extraction (gep/load per field) is identical across backends; they only
// DIVERGE on the element bounds check (same split as getter_array.sol):
//   - solx: explicit sol.length -> sol.cmp lt -> sol.require, then plain sol.gep.
//   - solc: `sol.gep ... no_panic_bounds`.

// CHECK-SOLX: sol.func @{{.*items.*}}(%arg0: ui256) -> (ui256, i1) attributes {{.*}}selector = -1078840878 : i32
// CHECK-SOLX:   %[[A:.*]] = sol.addr_of @{{.*items.*}} : !sol.array<? x !sol.struct<(ui256, i1), Storage>, Storage>
// CHECK-SOLX:   %[[LEN:.*]] = sol.length %[[A]] :
// CHECK-SOLX:   %[[OK:.*]] = sol.cmp lt, %arg0, %[[LEN]] : ui256
// CHECK-SOLX:   sol.require %[[OK]]()
// CHECK-SOLX:   %[[E:.*]] = sol.gep %[[A]], %arg0 : !sol.array<? x !sol.struct<(ui256, i1), Storage>, Storage>, ui256, !sol.struct<(ui256, i1), Storage>
// CHECK-SOLX:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK-SOLX:   %[[P0:.*]] = sol.gep %[[E]], %[[I0]] : !sol.struct<(ui256, i1), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK-SOLX:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-SOLX:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK-SOLX:   %[[P1:.*]] = sol.gep %[[E]], %[[I1]] : !sol.struct<(ui256, i1), Storage>, ui64, !sol.ptr<i1, Storage>
// CHECK-SOLX:   %[[V1:.*]] = sol.load %[[P1]] : !sol.ptr<i1, Storage>, i1
// CHECK-SOLX:   sol.return %[[V0]], %[[V1]] : ui256, i1

// CHECK-SOLC: sol.func @{{.*items.*}}(%arg0: ui256) -> (ui256, i1) attributes {{.*}}selector = -1078840878 : i32
// CHECK-SOLC:   %[[A:.*]] = sol.addr_of @{{.*items.*}} : !sol.array<? x !sol.struct<(ui256, i1), Storage>, Storage>
// CHECK-SOLC:   %[[E:.*]] = sol.gep %[[A]], %arg0 no_panic_bounds : !sol.array<? x !sol.struct<(ui256, i1), Storage>, Storage>, ui256, !sol.struct<(ui256, i1), Storage>
// CHECK-SOLC:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK-SOLC:   %[[P0:.*]] = sol.gep %[[E]], %[[I0]] : !sol.struct<(ui256, i1), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK-SOLC:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-SOLC:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK-SOLC:   %[[P1:.*]] = sol.gep %[[E]], %[[I1]] : !sol.struct<(ui256, i1), Storage>, ui64, !sol.ptr<i1, Storage>
// CHECK-SOLC:   %[[V1:.*]] = sol.load %[[P1]] : !sol.ptr<i1, Storage>, i1
// CHECK-SOLC:   sol.return %[[V0]], %[[V1]] : ui256, i1

contract C {
    struct Item { uint256 id; bool ok; }
    Item[] public items;
}
