// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #NonPayable
// CHECK:   sol.sig : !sol.fixedbytes<4>
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, !sol.ptr<!sol.fixedbytes<4>, Storage>

contract C {
    bytes4 public lastSig;
    fallback() external {
        lastSig = msg.sig;
    }
}
