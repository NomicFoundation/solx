// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Explicit `T(x)` conversions: widening a fixed-bytes value and reinterpreting
// `bytesN` as an integer both use `sol.bytes_cast`; narrowing a signed integer
// uses `sol.cast`. The functions emit in different orders (solx alphabetical,
// solc source), so match each distinct cast with CHECK-DAG.

// CHECK-DAG: sol.bytes_cast %{{.*}} : !sol.fixedbytes<4> to !sol.fixedbytes<32>
// CHECK-DAG: sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to ui256
// CHECK-DAG: sol.cast %{{.*}} : si256 to si8

contract C {
    function widen_bytes(bytes4 x) public pure returns (bytes32) { return bytes32(x); }
    function bytes_to_uint(bytes32 x) public pure returns (uint256) { return uint256(x); }
    function int_narrow(int256 x) public pure returns (int8) { return int8(x); }
}
