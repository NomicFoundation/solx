// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// User-defined `*` binary operator on a value type dispatches to the bound
// function via `sol.call`, not native arithmetic. Both backends emit `run`
// first and the call inside it; the bound symbol name diverges (solc: @f_<id>,
// solx: appends a NodeId), matched by regex.

// CHECK: sol.func @{{.*run.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK: sol.call @{{.*f.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.return %{{.*}} : ui256

type T is uint256;

function f(T a, T b) pure returns (T) {
    return T.wrap(T.unwrap(a) * T.unwrap(b));
}

using {f as *} for T global;

contract C {
    function run(T a, T b) public pure returns (T) {
        return a * b;
    }
}
