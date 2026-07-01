// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*g.*}}(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector = -467655094 : i32
// CHECK: sol.func @{{.*g.*}}(%{{.*}}: si256) -> si256 attributes {{.*}}selector = 2021111811 : i32
// CHECK: sol.func @{{.*g.*}}(%{{.*}}: i1) -> i1 attributes {{.*}}selector = -729771273 : i32

contract C {
    function g(uint256 a) public pure returns (uint256) {
        return a;
    }

    function g(int256 a) public pure returns (int256) {
        return a;
    }

    function g(bool a) public pure returns (bool) {
        return a;
    }
}
