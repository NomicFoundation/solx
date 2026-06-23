// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*bit_and.*}}
// CHECK-DAG:   sol.and %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*bit_or.*}}
// CHECK-DAG:   sol.or %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*bit_xor.*}}
// CHECK-DAG:   sol.xor %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*bit_not.*}}
// CHECK-DAG:   sol.not %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*shift_left.*}}
// CHECK-DAG:   sol.shl %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*shift_right.*}}
// CHECK-DAG:   sol.shr %{{.*}}, %{{.*}} : ui256

contract C {
    function bit_and(uint256 a, uint256 b) public pure returns (uint256) {
        return a & b;
    }

    function bit_or(uint256 a, uint256 b) public pure returns (uint256) {
        return a | b;
    }

    function bit_xor(uint256 a, uint256 b) public pure returns (uint256) {
        return a ^ b;
    }

    function bit_not(uint256 a) public pure returns (uint256) {
        return ~a;
    }

    function shift_left(uint256 a, uint256 b) public pure returns (uint256) {
        return a << b;
    }

    function shift_right(uint256 a, uint256 b) public pure returns (uint256) {
        return a >> b;
    }
}
