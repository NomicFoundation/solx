// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `sol.and`/`or`/`xor`/`shl`/`shr`/`not` are integer-only, but Solidity allows
// the bitwise operators on `bytesN`. Each fixed-bytes operand is bridged through
// the equivalent unsigned integer `ui(8*N)`, the op runs there, and the result
// is cast back to the fixed-bytes type. (solc emits the same bridge, but errors
// on the shift-amount width, so this checks solx only.)

// CHECK-DAG: sol.func @{{.*xr.*}}
// CHECK-DAG: sol.bytes_cast %{{[0-9]+}} : !sol.fixedbytes<4> to ui32
// CHECK-DAG: sol.bytes_cast %{{[0-9]+}} : !sol.fixedbytes<4> to ui32
// CHECK-DAG: sol.xor %{{[0-9]+}}, %{{[0-9]+}} : ui32
// CHECK-DAG: sol.bytes_cast %{{[0-9]+}} : ui32 to !sol.fixedbytes<4>

// CHECK-DAG: sol.func @{{.*bnot.*}}
// CHECK-DAG: sol.bytes_cast %{{[0-9]+}} : !sol.fixedbytes<32> to ui256
// CHECK-DAG: sol.not %{{[0-9]+}} : ui256
// CHECK-DAG: sol.bytes_cast %{{[0-9]+}} : ui256 to !sol.fixedbytes<32>

contract C {
    function xr(bytes4 a, bytes4 b) public pure returns (bytes4) {
        return a ^ b;
    }

    function bnot(bytes32 a) public pure returns (bytes32) {
        return ~a;
    }
}
