// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #NonPayable
// CHECK-DAG:   sol.get_calldata : !sol.string<CallData>
// CHECK-DAG:   sol.length %{{.*}} : !sol.string<CallData>
// CHECK-DAG:   sol.addr_of @{{.*lastLen.*}} : !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:       sol.return

contract C {
    uint256 public lastLen;
    fallback() external {
        lastLen = msg.data.length;
    }
}
