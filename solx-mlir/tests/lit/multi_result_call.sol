// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*pair.*}}
// CHECK: sol.return %{{.*}}, %{{.*}}
// CHECK: sol.func @{{.*sum_pair.*}}
// CHECK: %{{[0-9]+}}:2 = sol.call
// CHECK-NEXT: sol.alloca
// CHECK-NEXT: sol.store %{{[0-9]+}}#0
// CHECK-NEXT: sol.alloca
// CHECK-NEXT: sol.store %{{[0-9]+}}#1

contract C {
    function pair() public pure returns (uint256, uint256) {
        return (3, 7);
    }
    function sum_pair() public pure returns (uint256) {
        (uint256 a, uint256 b) = pair();
        return a + b;
    }
}
