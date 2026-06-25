// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Same-width signedness flips and a non-power-of-two width (uint32 -> uint64)
// all lower to a plain integer `sol.cast`. Both backends agree on the ops;
// only the function emission order differs, so CHECK-DAG is used.

// CHECK-DAG: sol.func @{{.*u2s.*}}(%{{.*}}: ui8) -> si8
// CHECK-DAG:   sol.cast %{{.*}} : ui8 to si8

// CHECK-DAG: sol.func @{{.*s2u.*}}(%{{.*}}: si8) -> ui8
// CHECK-DAG:   sol.cast %{{.*}} : si8 to ui8

// CHECK-DAG: sol.func @{{.*u32.*}}(%{{.*}}: ui32) -> ui64
// CHECK-DAG:   sol.cast %{{.*}} : ui32 to ui64

contract C {
    function u2s(uint8 a) public pure returns (int8) { return int8(a); }
    function s2u(int8 a) public pure returns (uint8) { return uint8(a); }
    function u32(uint32 a) public pure returns (uint64) { return uint64(a); }
}
