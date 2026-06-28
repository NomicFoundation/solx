// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// `sol.and`/`or`/`xor`/`shl`/`shr`/`not` are integer-only, but Solidity allows the
// bitwise operators on `bytesN`. solx bridges each fixed-bytes operand through the
// unsigned integer `ui(8*N)`: `bytes_cast` in, the integer op, `bytes_cast` back.
// solc keeps the op on the fixed-bytes type directly. CHECK-SOLX pins solx's bridge,
// CHECK-SOLC the native fixed-bytes op.

// CHECK-DAG: sol.func @{{.*xr.*}}
// CHECK-SOLX-DAG: sol.bytes_cast %{{.*}} : !sol.fixedbytes<4> to ui32
// CHECK-SOLX-DAG: sol.xor %{{.*}}, %{{.*}} : ui32
// CHECK-SOLX-DAG: sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK-SOLC-DAG: sol.xor %{{.*}}, %{{.*}} : !sol.fixedbytes<4>

// CHECK-DAG: sol.func @{{.*bnot.*}}
// CHECK-SOLX-DAG: sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to ui256
// CHECK-SOLX-DAG: sol.not %{{.*}} : ui256
// CHECK-SOLX-DAG: sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK-SOLC-DAG: sol.not %{{.*}} : !sol.fixedbytes<32>

contract C {
    function xr(bytes4 a, bytes4 b) public pure returns (bytes4) {
        return a ^ b;
    }

    function bnot(bytes32 a) public pure returns (bytes32) {
        return ~a;
    }
}
