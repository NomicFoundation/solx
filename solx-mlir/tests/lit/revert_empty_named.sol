// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A degenerate empty named-argument list `revert({})` is equivalent to `revert()`
// and lowers to a no-data revert, matching solc.
// CHECK: sol.revert ""

// FIX: can be merged intol revert.sol ?

contract C {
    function f(bool b) external pure {
        if (b) revert({});
    }
}
