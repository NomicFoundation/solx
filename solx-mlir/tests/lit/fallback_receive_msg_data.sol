// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// fallback() that reads msg.data.length and stores it. Pin the #Fallback /
// #NonPayable kind and the `lastLen = msg.data.length` body: both backends
// lower msg.data to `sol.get_calldata` + `sol.length`, then store to storage.
// solx emits get_calldata/length before the store addr_of, solc emits the
// addr_of first, so the body is matched order-independently with CHECK-DAG.

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
