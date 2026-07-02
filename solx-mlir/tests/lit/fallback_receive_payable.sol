// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #Payable
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.callvalue : ui256
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 public received;

    fallback() external payable {
        received = received + msg.value;
    }
}
