// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Compound assignment `s += y` to a value-typed STATE variable (the Storage
// assignment target): addr_of the slot, load the old value, load the rhs, combine
// with `sol.cadd`, store back to storage. Both backends emit this op set; they
// differ only in the order of the two loads (CHECK-DAG) and solx re-emits an
// addr_of for the store (so the final store's slot is matched loosely).

// CHECK: sol.func @{{.*f.*}}(%arg0: ui256)
// CHECK:   sol.addr_of @{{.*s.*}} : !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 s;
    function f(uint256 y) public { s += y; }
}
