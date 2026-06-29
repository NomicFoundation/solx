// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #NonPayable
// CHECK:   sol.get_calldata : !sol.string<CallData>
// CHECK:   sol.length %{{.*}} : !sol.string<CallData>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 public lastLength;

    fallback() external {
        lastLength = msg.data.length;
    }
}
