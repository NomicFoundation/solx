// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*bit_and.*}}
// CHECK:   sol.and %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*bit_and_signed.*}}
// CHECK:   sol.and %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @{{.*bit_and_fixed_bytes.*}}
// CHECK:   sol.and %{{.*}}, %{{.*}} : !sol.fixedbytes<4>

// CHECK: sol.func @{{.*bit_or.*}}
// CHECK:   sol.or %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*bit_or_signed.*}}
// CHECK:   sol.or %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @{{.*bit_or_fixed_bytes.*}}
// CHECK:   sol.or %{{.*}}, %{{.*}} : !sol.fixedbytes<4>

// CHECK: sol.func @{{.*bit_xor.*}}
// CHECK:   sol.xor %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*bit_xor_signed.*}}
// CHECK:   sol.xor %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @{{.*bit_xor_fixed_bytes.*}}
// CHECK:   sol.xor %{{.*}}, %{{.*}} : !sol.fixedbytes<4>

// CHECK: sol.func @{{.*bit_not.*}}
// CHECK:   sol.not %{{.*}} : ui256

// CHECK: sol.func @{{.*bit_not_signed.*}}
// CHECK:   sol.not %{{.*}} : si256

// CHECK: sol.func @{{.*bit_not_fixed_bytes.*}}
// CHECK:   sol.not %{{.*}} : !sol.fixedbytes<4>

// CHECK: sol.func @{{.*shift_left.*}}
// CHECK:   sol.shl %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*shift_left_mixed.*}}
// CHECK:   sol.shl %{{.*}}, %{{.*}} : ui8, ui256

// CHECK: sol.func @{{.*shift_left_fixed_bytes.*}}
// CHECK:   sol.shl %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, ui8

// CHECK: sol.func @{{.*shift_right.*}}
// CHECK:   sol.shr %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*shift_right_mixed.*}}
// CHECK:   sol.shr %{{.*}}, %{{.*}} : si8, ui256

// CHECK: sol.func @{{.*shift_right_fixed_bytes.*}}
// CHECK:   sol.shr %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, ui8

contract C {
    function bit_and(uint256 a, uint256 b) public pure returns (uint256) {
        return a & b;
    }

    function bit_and_signed(int256 a, int256 b) public pure returns (int256) {
        return a & b;
    }

    function bit_and_fixed_bytes(bytes4 a, bytes4 b) public pure returns (bytes4) {
        return a & b;
    }

    function bit_or(uint256 a, uint256 b) public pure returns (uint256) {
        return a | b;
    }

    function bit_or_signed(int256 a, int256 b) public pure returns (int256) {
        return a | b;
    }

    function bit_or_fixed_bytes(bytes4 a, bytes4 b) public pure returns (bytes4) {
        return a | b;
    }

    function bit_xor(uint256 a, uint256 b) public pure returns (uint256) {
        return a ^ b;
    }

    function bit_xor_signed(int256 a, int256 b) public pure returns (int256) {
        return a ^ b;
    }

    function bit_xor_fixed_bytes(bytes4 a, bytes4 b) public pure returns (bytes4) {
        return a ^ b;
    }

    function bit_not(uint256 a) public pure returns (uint256) {
        return ~a;
    }

    function bit_not_signed(int256 a) public pure returns (int256) {
        return ~a;
    }

    function bit_not_fixed_bytes(bytes4 a) public pure returns (bytes4) {
        return ~a;
    }

    function shift_left(uint256 value, uint256 amount) public pure returns (uint256) {
        return value << amount;
    }

    function shift_left_mixed(uint8 value, uint256 amount) public pure returns (uint8) {
        return value << amount;
    }

    function shift_left_fixed_bytes(bytes4 value, uint8 amount) public pure returns (bytes4) {
        return value << amount;
    }

    function shift_right(uint256 value, uint256 amount) public pure returns (uint256) {
        return value >> amount;
    }

    function shift_right_mixed(int8 value, uint256 amount) public pure returns (int8) {
        return value >> amount;
    }

    function shift_right_fixed_bytes(bytes4 value, uint8 amount) public pure returns (bytes4) {
        return value >> amount;
    }
}
