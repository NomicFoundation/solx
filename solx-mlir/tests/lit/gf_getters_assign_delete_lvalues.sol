// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `delete` on VALUE-typed lvalues (not the reference-aggregate case covered by
// delete.sol, which uses solx's `sol.delete`). For a value lvalue both backends
// agree: reset it to its typed zero (`sol.constant 0` then `sol.store`). Covered
// here for the three Pointer/Storage lvalue kinds — local, array element, struct
// field. Functions are emitted alphabetically by solx / in source order by solc,
// so this file keeps that order identical (delField, delElem, delLocal) and the
// source matches; each block is anchored by its own regex symbol.

// CHECK: sol.func @{{.*delElem.*}}(%arg0: ui256)
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*arr.*}} : !sol.array<? x ui256, Storage>
// CHECK:   %[[P:.*]] = sol.gep %[[A]], %{{.*}} : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   %[[Z:.*]] = sol.constant 0 : ui256
// CHECK:   sol.store %[[Z]], %[[P]] : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*delField.*}}()
// CHECK:   sol.addr_of @{{.*s.*}} : !sol.struct<(ui256, ui256), Storage>
// CHECK:   %[[FP:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[FZ:.*]] = sol.constant 0 : ui256
// CHECK:   sol.store %[[FZ]], %[[FP]] : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*delLocal.*}}(%arg0: ui256)
// CHECK:   %[[L:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.store %arg0, %[[L]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   %[[LZ:.*]] = sol.constant 0 : ui256
// CHECK:   sol.store %[[LZ]], %[[L]] : ui256, !sol.ptr<ui256, Stack>

contract C {
    struct S { uint256 a; uint256 b; }
    S s;
    uint256[] arr;

    function delElem(uint256 i) public { delete arr[i]; }
    function delField() public { delete s.b; }
    function delLocal(uint256 x) public pure { delete x; }
}
