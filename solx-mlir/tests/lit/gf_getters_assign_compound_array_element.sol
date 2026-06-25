// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Compound assignment `arr[i] += y` to a storage array element (the Pointer
// assignment target reached through an index lvalue): addr_of the array, gep the
// element pointer, load old + rhs, `sol.cadd`, then store back into the SAME gep'd
// pointer. Both backends share this; only the two loads are reordered (CHECK-DAG).

// CHECK: sol.func @{{.*f.*}}(%arg0: ui256, %arg1: ui256)
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*arr.*}} : !sol.array<? x ui256, Storage>
// CHECK:   %[[P:.*]] = sol.gep %[[A]], %{{.*}} : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[R:.*]] = sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %[[R]], %[[P]] : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256[] arr;
    function f(uint256 i, uint256 y) public { arr[i] += y; }
}
