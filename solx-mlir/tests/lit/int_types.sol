// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*bool_false.*}}
// CHECK:   %false = sol.constant false

// CHECK: sol.func @{{.*bool_return.*}}
// CHECK:   %true = sol.constant true

// CHECK: sol.func @{{.*int256_arithmetic.*}}(%{{.*}}: si256, %{{.*}}: si256) -> si256
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @{{.*uint128_arithmetic.*}}(%{{.*}}: ui128, %{{.*}}: ui128) -> ui128
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui128

// CHECK: sol.func @{{.*uint8_arithmetic.*}}(%{{.*}}: ui8, %{{.*}}: ui8) -> ui8
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui8

contract C {
    function bool_false() public pure returns (bool) {
        return false;
    }

    function bool_return() public pure returns (bool) {
        return true;
    }

    function int256_arithmetic(int256 a, int256 b) public pure returns (int256) {
        return a + b;
    }

    function uint128_arithmetic(uint128 a, uint128 b) public pure returns (uint128) {
        return a + b;
    }

    function uint8_arithmetic(uint8 a, uint8 b) public pure returns (uint8) {
        return a + b;
    }
}
