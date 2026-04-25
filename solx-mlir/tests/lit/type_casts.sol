// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"uint8_to_uint256(uint8)"
// CHECK:   sol.cast %{{.*}} : ui8 to ui256

// CHECK: sol.func @"uint256_to_uint8(uint256)"
// CHECK:   sol.cast %{{.*}} : ui256 to ui8

// CHECK: sol.func @"int_to_uint(int256)"
// CHECK:   sol.cast %{{.*}} : si256 to ui256

// CHECK: sol.func @"uint_to_bool(uint256)"
// CHECK:   sol.cmp ne, %{{.*}}, %{{.*}} : ui256

contract C {
    function uint8_to_uint256(uint8 x) public pure returns (uint256) {
        return uint256(x);
    }

    function uint256_to_uint8(uint256 x) public pure returns (uint8) {
        return uint8(x);
    }

    function int_to_uint(int256 x) public pure returns (uint256) {
        return uint256(x);
    }

    function uint_to_bool(uint256 x) public pure returns (bool) {
        return x != 0;
    }
}
