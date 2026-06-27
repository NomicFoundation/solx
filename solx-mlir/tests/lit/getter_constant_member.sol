// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Auto-generated getter for a `public constant` whose initializer is a
// cross-namespace member access (`Lib.BASE`). solx resolves the member to its
// constant definition and folds it into a single `sol.constant` of the result
// type, returned `#Pure`. solc's MLIR backend aborts on a member-access
// constant initializer (UNREACHABLE in SolidityToMLIR), so this is solx-only.

// CHECK-LABEL: sol.func @"DERIVED()"() -> ui256 attributes {{.*}}selector = 1646776813 : i32{{.*}}#Pure
// CHECK:   %[[C:.*]] = sol.constant 7 : ui256
// CHECK:   sol.return %[[C]] : ui256

library Lib {
    uint256 internal constant BASE = 7;
}

contract C {
    uint256 public constant DERIVED = Lib.BASE;
}
