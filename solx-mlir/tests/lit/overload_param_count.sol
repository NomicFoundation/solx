// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector = -1277270901 : i32
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*f.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256 attributes {{.*}}selector = 332507694 : i32
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256

contract C {
    function f(uint256 a) public pure returns (uint256) {
        return a + 1;
    }

    function f(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}
