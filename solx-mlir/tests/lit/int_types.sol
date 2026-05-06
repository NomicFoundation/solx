// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*uint8_arith.*}}(%{{.*}}: ui8, %{{.*}}: ui8) -> ui8
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui8

// CHECK: sol.func @{{.*uint128_arith.*}}(%{{.*}}: ui128, %{{.*}}: ui128) -> ui128
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui128

// CHECK: sol.func @{{.*int256_arith.*}}(%{{.*}}: si256, %{{.*}}: si256) -> si256
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @{{.*bool_return.*}}
// CHECK:   %true = arith.constant true

// CHECK: sol.func @{{.*bool_false.*}}
// CHECK:   %false = arith.constant false

contract C {
    function uint8_arith(uint8 a, uint8 b) public pure returns (uint8) {
        return a + b;
    }

    function uint128_arith(uint128 a, uint128 b) public pure returns (uint128) {
        return a + b;
    }

    function int256_arith(int256 a, int256 b) public pure returns (int256) {
        return a + b;
    }

    function bool_return() public pure returns (bool) {
        return true;
    }

    function bool_false() public pure returns (bool) {
        return false;
    }
}
