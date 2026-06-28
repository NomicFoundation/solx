// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*uint8_to_uint256.*}}
// CHECK-DAG:   sol.cast %{{.*}} : ui8 to ui256

// CHECK-DAG: sol.func @{{.*uint256_to_uint8.*}}
// CHECK-DAG:   sol.cast %{{.*}} : ui256 to ui8

// CHECK-DAG: sol.func @{{.*int_to_uint.*}}
// CHECK-DAG:   sol.cast %{{.*}} : si256 to ui256

// CHECK-DAG: sol.func @{{.*uint_to_bool.*}}
// CHECK-DAG:   sol.cmp ne, %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*int_widen.*}}
// CHECK-DAG:   sol.cast %{{.*}} : si8 to si256

// CHECK-DAG: sol.func @{{.*int_narrow.*}}
// CHECK-DAG:   sol.cast %{{.*}} : si256 to si16

// CHECK-DAG: sol.func @{{.*sign_to_signed.*}}
// CHECK-DAG:   sol.cast %{{.*}} : ui8 to si8

// CHECK-DAG: sol.func @{{.*sign_to_unsigned.*}}
// CHECK-DAG:   sol.cast %{{.*}} : si8 to ui8

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

    function int_widen(int8 x) public pure returns (int256) {
        return int256(x);
    }

    function int_narrow(int256 x) public pure returns (int16) {
        return int16(x);
    }

    function sign_to_signed(uint8 x) public pure returns (int8) {
        return int8(x);
    }

    function sign_to_unsigned(int8 x) public pure returns (uint8) {
        return uint8(x);
    }
}
