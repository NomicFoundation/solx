// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `sol.cast` is integer-only; conversions touching fixedbytes / address / enum
// dispatch to the dedicated cast op (centralised in Builder::emit_sol_cast), so
// they never reach `sol.cast`'s integer-only folder.

// CHECK-DAG: sol.func @{{.*to_bytes32.*}}
// CHECK-DAG: sol.bytes_cast %{{.*}} to !sol.fixedbytes<32>

// CHECK-DAG: sol.func @{{.*narrow_bytes.*}}
// CHECK-DAG: sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to !sol.fixedbytes<16>

// CHECK-DAG: sol.func @{{.*to_address.*}}
// CHECK-DAG: sol.address_cast %{{.*}} to !sol.address

// CHECK-DAG: sol.func @{{.*from_enum.*}}
// CHECK-DAG: sol.enum_cast %{{.*}} : !sol.enum<2> to ui8

// CHECK-DAG: sol.func @{{.*to_enum.*}}
// CHECK-DAG: sol.enum_cast %{{.*}} : ui8 to !sol.enum<2>

contract C {
    enum E {
        A,
        B,
        C
    }

    function to_bytes32(uint256 x) public pure returns (bytes32) {
        return bytes32(x);
    }

    function narrow_bytes(bytes32 x) public pure returns (bytes16) {
        return bytes16(x);
    }

    function to_address(uint160 x) public pure returns (address) {
        return address(x);
    }

    function from_enum(E e) public pure returns (uint8) {
        return uint8(e);
    }

    function to_enum(uint8 x) public pure returns (E) {
        return E(x);
    }
}
