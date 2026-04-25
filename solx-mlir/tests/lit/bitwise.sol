// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"bit_and(uint256,uint256)"
// CHECK:   sol.and %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"bit_or(uint256,uint256)"
// CHECK:   sol.or %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"bit_xor(uint256,uint256)"
// CHECK:   sol.xor %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"bit_not(uint256)"
// CHECK:   sol.xor %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"shift_left(uint256,uint256)"
// CHECK:   sol.shl %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"shift_right(uint256,uint256)"
// CHECK:   sol.shr %{{.*}}, %{{.*}} : ui256

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
