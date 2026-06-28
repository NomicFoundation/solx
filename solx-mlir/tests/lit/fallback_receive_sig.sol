// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #NonPayable
// CHECK-DAG:   sol.sig : !sol.fixedbytes<4>
// CHECK-DAG:   sol.addr_of @{{.*lastSig.*}} : !sol.ptr<!sol.fixedbytes<4>, Storage>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, !sol.ptr<!sol.fixedbytes<4>, Storage>
// CHECK:       sol.return

contract C {
    bytes4 public lastSig;
    fallback() external {
        lastSig = msg.sig;
    }
}
