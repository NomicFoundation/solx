// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Auto-generated getter for a `public constant` whose initializer IS an
// integer-foldable form (a decimal literal). solx constant-folds the initializer
// into a single `sol.constant <value> : ui256` of the result type and returns it
// (state_mutability #Pure). solc emits the literal at its narrow literal type
// (ui8) plus an explicit `sol.cast` widening to ui256, mutability #NonPayable.
// Genuine benign divergence (fold-vs-cast + mutability) -> split prefixes.
// Both pin the same selector. solx names it `LIMIT()`, solc `get_LIMIT_<id>`.

// CHECK-SOLX: sol.func @{{.*LIMIT.*}}() -> ui256 attributes {{.*}}selector = -1350429457 : i32{{.*}}#Pure
// CHECK-SOLX:   %[[C:.*]] = sol.constant 42 : ui256
// CHECK-SOLX:   sol.return %[[C]] : ui256

// CHECK-SOLC: sol.func @{{.*LIMIT.*}}() -> ui256 attributes {{.*}}selector = -1350429457 : i32
// CHECK-SOLC:   %[[C:.*]] = sol.constant 42 : ui8
// CHECK-SOLC:   %[[V:.*]] = sol.cast %[[C]] : ui8 to ui256
// CHECK-SOLC:   sol.return %[[V]] : ui256

contract C {
    uint256 public constant LIMIT = 42;
}
