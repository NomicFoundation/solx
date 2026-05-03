// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*sum.*}}
// Parameters get their own alloca + store from %arg.
// CHECK: sol.store %arg0
// CHECK: sol.store %arg1
// CHECK: sol.store %arg2
// Then loads (the RHS tuple values) come before the deconstructed allocas.
// CHECK: sol.load
// CHECK-NEXT: sol.load
// CHECK-NEXT: sol.load
// Each deconstructed local: alloca followed by a store of a loaded value
// (not %arg), proving the deconstruction allocated fresh slots for a, b, c.
// CHECK: sol.alloca
// CHECK-NEXT: sol.store %{{[0-9]+}}
// CHECK: sol.alloca
// CHECK-NEXT: sol.store %{{[0-9]+}}
// CHECK: sol.alloca
// CHECK-NEXT: sol.store %{{[0-9]+}}

contract C {
    function sum(uint256 x, uint256 y, uint256 z) public pure returns (uint256) {
        (uint256 a, uint256 b, uint256 c) = (x, y, z);
        return a + b + c;
    }
}
