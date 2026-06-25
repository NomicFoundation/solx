// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// payable fallback() that accumulates msg.value into state. Pin the #Fallback /
// #Payable function kind and the `received += msg.value` body. As with the
// payable receive, both backends emit callvalue/addr_of/load/cadd/store but
// order them differently, so the body is matched with CHECK-DAG.

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #Payable
// CHECK-DAG:   sol.callvalue : ui256
// CHECK-DAG:   sol.addr_of @{{.*received.*}} : !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK-DAG:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:       sol.return

contract C {
    uint256 public received;
    fallback() external payable {
        received = received + msg.value;
    }
}
