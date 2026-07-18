// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*uint8_to_uint256.*}}
// CHECK:   sol.cast %{{.*}} : ui8 to ui256

// CHECK: sol.func @{{.*uint256_to_uint8.*}}
// CHECK:   sol.cast %{{.*}} : ui256 to ui8

// CHECK: sol.func @{{.*int_to_uint.*}}
// CHECK:   sol.cast %{{.*}} : si256 to ui256

// CHECK: sol.func @{{.*bytes4_to_int.*}}
// CHECK:   sol.bytes_cast %{{.*}} : !sol.fixedbytes<4> to ui32

// CHECK: sol.func @{{.*bytes_to_int.*}}
// CHECK:   sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to ui256

// CHECK: sol.func @{{.*int_to_bytes.*}}
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>

// CHECK: sol.func @{{.*widen_bytes.*}}
// CHECK:   sol.bytes_cast %{{.*}} : !sol.fixedbytes<1> to !sol.fixedbytes<4>

// CHECK: sol.func @{{.*narrow_bytes.*}}
// CHECK:   sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to !sol.fixedbytes<16>

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

    function bytes4_to_int(bytes4 x) public pure returns (uint32) {
        return uint32(x);
    }

    function bytes_to_int(bytes32 x) public pure returns (uint256) {
        return uint256(x);
    }

    function int_to_bytes(uint256 x) public pure returns (bytes32) {
        return bytes32(x);
    }

    function widen_bytes(bytes1 x) public pure returns (bytes4) {
        return bytes4(x);
    }

    function narrow_bytes(bytes32 x) public pure returns (bytes16) {
        return bytes16(x);
    }
}
