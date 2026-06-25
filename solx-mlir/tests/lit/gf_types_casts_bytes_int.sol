// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// fixed-bytes <-> integer and fixed-bytes <-> fixed-bytes conversions lower to
// `sol.bytes_cast`. When the integer width equals the bytes partner width
// (bytes32<->ui256, bytes4<->ui32) it is a single direct `bytes_cast`. A
// fixed-bytes -> wider integer (bytes1 -> uint8 -> uint256) emits a
// `bytes_cast` to the partner integer (ui8) followed by an integer `sol.cast`.
// fixed-bytes -> fixed-bytes (bytes1 -> bytes4) is a single `bytes_cast`.
// Both backends agree; function order differs so CHECK-DAG is used.

// CHECK-DAG: sol.func @{{.*b32_to_u.*}}(%{{.*}}: !sol.fixedbytes<32>) -> ui256
// CHECK-DAG:   sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to ui256

// CHECK-DAG: sol.func @{{.*u_to_b32.*}}(%{{.*}}: ui256) -> !sol.fixedbytes<32>
// CHECK-DAG:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>

// CHECK-DAG: sol.func @{{.*b4_to_u.*}}(%{{.*}}: !sol.fixedbytes<4>) -> ui32
// CHECK-DAG:   sol.bytes_cast %{{.*}} : !sol.fixedbytes<4> to ui32

// CHECK-DAG: sol.func @{{.*b1_to_b4.*}}(%{{.*}}: !sol.fixedbytes<1>) -> !sol.fixedbytes<4>
// CHECK-DAG:   sol.bytes_cast %{{.*}} : !sol.fixedbytes<1> to !sol.fixedbytes<4>

contract C {
    function b32_to_u(bytes32 b) public pure returns (uint256) { return uint256(b); }
    function u_to_b32(uint256 u) public pure returns (bytes32) { return bytes32(u); }
    function b4_to_u(bytes4 b) public pure returns (uint32) { return uint32(b); }
    function b1_to_b4(bytes1 b) public pure returns (bytes4) { return bytes4(b); }
}
