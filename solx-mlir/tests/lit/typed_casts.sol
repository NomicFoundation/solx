// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*bytes4_to_int.*}}
// CHECK: sol.bytes_cast %{{.*}} : !sol.fixedbytes<4> to ui32

// CHECK: sol.func @{{.*bytes_to_int.*}}
// CHECK: sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to ui256

// CHECK: sol.func @{{.*from_enum.*}}
// CHECK: sol.enum_cast %{{.*}} : !sol.enum<2> to ui8

// CHECK: sol.func @{{.*narrow_bytes.*}}
// CHECK: sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to !sol.fixedbytes<16>

// CHECK: sol.func @{{.*to_address.*}}
// CHECK: sol.address_cast %{{.*}} to !sol.address

// CHECK: sol.func @{{.*to_bytes32.*}}
// CHECK: sol.bytes_cast %{{.*}} to !sol.fixedbytes<32>

// CHECK: sol.func @{{.*to_enum.*}}
// CHECK: sol.enum_cast %{{.*}} : ui8 to !sol.enum<2>

// CHECK: sol.func @{{.*widen_bytes.*}}
// CHECK: sol.bytes_cast %{{.*}} : !sol.fixedbytes<1> to !sol.fixedbytes<4>

contract C {
    enum E {
        A,
        B,
        C
    }

    function bytes4_to_int(bytes4 x) public pure returns (uint32) {
        return uint32(x);
    }

    function bytes_to_int(bytes32 x) public pure returns (uint256) {
        return uint256(x);
    }

    function from_enum(E e) public pure returns (uint8) {
        return uint8(e);
    }

    function narrow_bytes(bytes32 x) public pure returns (bytes16) {
        return bytes16(x);
    }

    function to_address(uint160 x) public pure returns (address) {
        return address(x);
    }

    function to_bytes32(uint256 x) public pure returns (bytes32) {
        return bytes32(x);
    }

    function to_enum(uint8 x) public pure returns (E) {
        return E(x);
    }

    function widen_bytes(bytes1 x) public pure returns (bytes4) {
        return bytes4(x);
    }
}
