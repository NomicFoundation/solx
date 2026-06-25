// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Compound assignment `s.b *= y` to a storage struct FIELD (the Pointer target
// reached through a member-access lvalue): addr_of the struct, gep field 1, load
// old + rhs, `sol.cmul`, store back into the same field pointer. Both backends
// agree on the op set; the two loads and the addr_of/constant ordering differ, so
// the field pointer is re-captured from the gep and the loads are CHECK-DAG.

// CHECK: sol.func @{{.*f.*}}(%arg0: ui256)
// CHECK:   sol.addr_of @{{.*s.*}} : !sol.struct<(ui256, ui256), Storage>
// CHECK:   %[[P:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[R:.*]] = sol.cmul %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %[[R]], %[[P]] : ui256, !sol.ptr<ui256, Storage>

contract C {
    struct S { uint256 a; uint256 b; }
    S s;
    function f(uint256 y) public { s.b *= y; }
}
