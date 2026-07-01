// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Named arguments on a custom error inside `require(...)` are reordered into the
// error's declaration order ({b: 99, a: 11} -> (a, b)) and lowered to the call form
// of `sol.require`, byte-matching solc.
// CHECK: sol.require %{{.*}}, "MyErr(uint256,uint256)"({{.*}}) {call}

contract C {
    error MyErr(uint256 a, uint256 b);

    function f(bool ok) external pure {
        require(ok, MyErr({b: 99, a: 11}));
    }
}
