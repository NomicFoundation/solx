// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Shift (`<<` / `>>`) of a `bytesN` value. `sol.shl` / `sol.shr` are integer-only
// in solx's emitter, so a fixed-bytes operand is bridged through `ui(8*N)`:
// the shifted value is `bytes_cast` to the integer, the shift amount cast to
// the same width, the integer `sol.shl` / `sol.shr` applied, and the result
// `bytes_cast` back to `bytesN`. solc's nascent MLIR backend instead keeps
// `sol.shl` / `sol.shr` directly on `!sol.fixedbytes<N>`; behavioural parity is
// covered by the tester, so the bridge vs direct-op shape is checked per-tool.

// CHECK-SOLX: sol.func @{{.*shl.*}}
// CHECK-SOLX: %[[V:.*]] = sol.bytes_cast %{{[0-9]+}} : !sol.fixedbytes<4> to ui32
// CHECK-SOLX: %[[R:.*]] = sol.shl %[[V]], %{{[0-9]+}} : ui32, ui32
// CHECK-SOLX: sol.bytes_cast %[[R]] : ui32 to !sol.fixedbytes<4>

// CHECK-SOLX: sol.func @{{.*shr.*}}
// CHECK-SOLX: %[[V2:.*]] = sol.bytes_cast %{{[0-9]+}} : !sol.fixedbytes<4> to ui32
// CHECK-SOLX: %[[R2:.*]] = sol.shr %[[V2]], %{{[0-9]+}} : ui32, ui32
// CHECK-SOLX: sol.bytes_cast %[[R2]] : ui32 to !sol.fixedbytes<4>

// CHECK-SOLC: sol.func @{{.*shl.*}}
// CHECK-SOLC: sol.shl %{{[0-9]+}}, %{{[0-9]+}} : !sol.fixedbytes<4>, ui8
// CHECK-SOLC: sol.func @{{.*shr.*}}
// CHECK-SOLC: sol.shr %{{[0-9]+}}, %{{[0-9]+}} : !sol.fixedbytes<4>, ui8

// EXPLAIN: why different solc solx

contract C {
    function shl(bytes4 a, uint8 n) public pure returns (bytes4) { return a << n; }
    function shr(bytes4 a, uint8 n) public pure returns (bytes4) { return a >> n; }
}
