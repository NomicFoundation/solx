// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Integer width conversions route through `sol.cast` (integer-only). Both
// signed and unsigned, widening and narrowing, lower identically in both
// backends. Only function order differs (solx alphabetical, solc source order),
// so CHECK-DAG tolerates the swap.

// CHECK-DAG: sol.func @{{.*widen_u.*}}(%{{.*}}: ui8) -> ui256
// CHECK-DAG:   sol.cast %{{.*}} : ui8 to ui256

// CHECK-DAG: sol.func @{{.*narrow_u.*}}(%{{.*}}: ui256) -> ui8
// CHECK-DAG:   sol.cast %{{.*}} : ui256 to ui8

// CHECK-DAG: sol.func @{{.*widen_s.*}}(%{{.*}}: si8) -> si256
// CHECK-DAG:   sol.cast %{{.*}} : si8 to si256

// CHECK-DAG: sol.func @{{.*narrow_s.*}}(%{{.*}}: si256) -> si16
// CHECK-DAG:   sol.cast %{{.*}} : si256 to si16

contract C {
    function widen_u(uint8 a) public pure returns (uint256) { return uint256(a); }
    function narrow_u(uint256 a) public pure returns (uint8) { return uint8(a); }
    function widen_s(int8 a) public pure returns (int256) { return int256(a); }
    function narrow_s(int256 a) public pure returns (int16) { return int16(a); }
}
