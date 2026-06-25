// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// User-defined unary `~` (bitwise not) on a value type dispatches to the bound
// `bnot` function via `sol.call`, not native `sol.not`. solc names the bound
// function @bnot_<id>, solx appends a NodeId.

// CHECK: sol.func @{{.*run.*}}(%{{.*}}: ui256) -> ui256
// CHECK: sol.call @{{.*bnot.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.return %{{.*}} : ui256

type T is uint256;

function bnot(T a) pure returns (T) {
    return T.wrap(~T.unwrap(a));
}

using {bnot as ~} for T global;

contract C {
    function run(T a) public pure returns (T) {
        return ~a;
    }
}
