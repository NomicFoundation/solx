// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Three functions named `g` overloaded by parameter type (uint256 / int256 /
// bool). Each overload gets a distinct ABI selector and a distinct argument
// type, so resolution is purely type-driven. Both backends agree on the
// selectors and the lowered signatures; symbol names differ only by solc's
// _<nodeid> suffix. CHECK-DAG matches each overload independently of order.

// CHECK-DAG: sol.func @{{.*g.*}}(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector = -467655094 : i32
// CHECK-DAG: sol.func @{{.*g.*}}(%{{.*}}: si256) -> si256 attributes {{.*}}selector = 2021111811 : i32
// CHECK-DAG: sol.func @{{.*g.*}}(%{{.*}}: i1) -> i1 attributes {{.*}}selector = -729771273 : i32

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
