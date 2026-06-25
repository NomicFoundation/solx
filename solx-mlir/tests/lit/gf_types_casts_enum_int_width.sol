// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// enum is integer-backed at ui8 (`!sol.enum<N>` where N is the max enumerator).
// Converting an enum to a wider integer (uint256) emits a `sol.enum_cast` to
// the ui8 partner followed by an integer `sol.cast` widen ui8 -> ui256. The
// reverse uint8 -> enum is a single `sol.enum_cast`. Both backends agree;
// function order differs so CHECK-DAG is used.

// CHECK-DAG: sol.func @{{.*toU256.*}}(%{{.*}}: !sol.enum<2>) -> ui256
// CHECK-DAG:   sol.enum_cast %{{.*}} : !sol.enum<2> to ui8
// CHECK-DAG:   sol.cast %{{.*}} : ui8 to ui256

// CHECK-DAG: sol.func @{{.*fromU8.*}}(%{{.*}}: ui8) -> !sol.enum<2>
// CHECK-DAG:   sol.enum_cast %{{.*}} : ui8 to !sol.enum<2>

contract C {
    enum E { A, B, C }
    function toU256(E e) public pure returns (uint256) { return uint256(uint8(e)); }
    function fromU8(uint8 v) public pure returns (E) { return E(v); }
}
