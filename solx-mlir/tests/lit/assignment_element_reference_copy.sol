// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*setElem.*}}
// CHECK: %[[ARR:.*]] = sol.addr_of @{{arr.*}} : !sol.array<? x !sol.struct<(ui256, ui256), Storage>, Storage>
// CHECK: %[[ELEM:.*]] = sol.gep %[[ARR]], %{{.*}} {{.*}}!sol.struct<(ui256, ui256), Storage>
// CHECK: %[[SRC:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<!sol.struct<(ui256, ui256), Memory>, Stack>, !sol.struct<(ui256, ui256), Memory>
// CHECK: sol.copy %[[SRC]], %[[ELEM]] : !sol.struct<(ui256, ui256), Memory>, !sol.struct<(ui256, ui256), Storage>

contract C {
    struct S { uint256 a; uint256 b; }
    S[] arr;
    function setElem(S memory v) public { arr[0] = v; }
}
