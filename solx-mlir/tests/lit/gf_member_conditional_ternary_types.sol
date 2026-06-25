// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Ternary with mixed literal branches: the slot adopts the common literal type
// (ui8), the join loads it and casts to the declared ui256 return type. A string
// ternary routes each branch through the memory representation (sol.string_lit
// stored into a !sol.string<Memory> slot). solx walks alphabetically (lits, pick),
// solc in source order (lits, pick) — same order, so plain CHECK-LABEL blocks work.
// Symbols carry a solc node-id suffix (regex).

// CHECK-LABEL: sol.func @{{.*lits.*}}(%{{.*}}: i1) -> ui256
// CHECK: %[[S:.*]] = sol.alloca : !sol.ptr<ui8, Stack>
// CHECK: sol.if %{{.*}} {
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.store %{{.*}}, %[[S]] : ui8, !sol.ptr<ui8, Stack>
// CHECK: } else {
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.store %{{.*}}, %[[S]] : ui8, !sol.ptr<ui8, Stack>
// CHECK: }
// CHECK: %[[L:.*]] = sol.load %[[S]] : !sol.ptr<ui8, Stack>, ui8
// CHECK: sol.cast %[[L]] : ui8 to ui256

// CHECK-LABEL: sol.func @{{.*pick.*}}(%{{.*}}: i1) -> !sol.string<Memory>
// CHECK: %[[SS:.*]] = sol.alloca : !sol.ptr<!sol.string<Memory>, Stack>
// CHECK: sol.if %{{.*}} {
// CHECK:   sol.string_lit "yes" -> !sol.string<Memory>
// CHECK:   sol.store %{{.*}}, %[[SS]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>
// CHECK: } else {
// CHECK:   sol.string_lit "no" -> !sol.string<Memory>
// CHECK:   sol.store %{{.*}}, %[[SS]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>
// CHECK: }
// CHECK: sol.load %[[SS]] : !sol.ptr<!sol.string<Memory>, Stack>, !sol.string<Memory>

contract C {
    function lits(bool c) public pure returns (uint256) { return c ? 1 : 2; }
    function pick(bool c) public pure returns (string memory) { return c ? "yes" : "no"; }
}
