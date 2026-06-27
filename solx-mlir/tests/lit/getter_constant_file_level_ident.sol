// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Auto-generated getter for a `public constant` whose initializer is a bare
// identifier referencing a file-level `constant`. fold_constant_int recurses
// through the Definition::Constant reference and folds `A` to 7, so the getter
// returns a single `sol.constant 7 : ui256` (state_mutability #Pure). Reading a
// file-level constant as a runtime value aborts solc (getLocalVarAddr
// assertion), so this is a solx-only check.

// CHECK: sol.func @{{.*B.*}}() -> ui256 attributes {{.*}}selector = 854050239 : i32{{.*}}#Pure
// CHECK:   %[[C:.*]] = sol.constant 7 : ui256
// CHECK:   sol.return %[[C]] : ui256

uint256 constant A = 7;

contract C {
    uint256 public constant B = A;
}
