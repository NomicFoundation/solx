// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// receive() that updates state from msg.value: pin the #Receive / #Payable
// function kind and the `total += msg.value` body. Both backends emit the same
// ops (callvalue, addr_of/load/cadd/store) but in a different order: solx hoists
// callvalue first, solc emits the store-destination addr_of first. The body ops
// are matched order-independently with CHECK-DAG inside the receive function.

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Receive{{.*}}state_mutability = #Payable
// CHECK-DAG:   sol.callvalue : ui256
// CHECK-DAG:   sol.addr_of @{{.*total.*}} : !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK-DAG:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:       sol.return

contract C {
    uint256 public total;
    receive() external payable {
        total = total + msg.value;
    }
}
