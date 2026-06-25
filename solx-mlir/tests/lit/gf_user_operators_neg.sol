// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// User-defined unary `-` (negation) on a value type dispatches to the bound
// `neg` function via `sol.call`, not native `sol.sub` against zero. The
// using-operator mapping splits `-` into Neg vs Sub by arity (arity 1 -> Neg).
// solc names the bound function @neg_<id>, solx appends a NodeId.

// CHECK: sol.func @{{.*run.*}}(%{{.*}}: ui256) -> ui256
// CHECK: sol.call @{{.*neg.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.return %{{.*}} : ui256

type T is uint256;

function neg(T a) pure returns (T) {
    return T.wrap(0 - T.unwrap(a));
}

using {neg as -} for T global;

contract C {
    function run(T a) public pure returns (T) {
        return -a;
    }
}
