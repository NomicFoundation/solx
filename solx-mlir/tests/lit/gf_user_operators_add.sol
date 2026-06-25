// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// User-defined `+` on a value type. `a + b` dispatches to the bound `add`
// function via `sol.call`, not native `sol.cadd`. Both backends emit `run`
// before the operator function (solx: alphabetical, add < run is false but the
// bound function is appended last; solc: source order), so the call is pinned
// inside `run`. solc names the bound function @add_<id>, solx appends a NodeId.

// CHECK: sol.func @{{.*run.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK: sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.return %{{.*}} : ui256

type T is uint256;

function add(T a, T b) pure returns (T) {
    return T.wrap(T.unwrap(a) + T.unwrap(b));
}

using {add as +} for T global;

contract C {
    function run(T a, T b) public pure returns (T) {
        return a + b;
    }
}
