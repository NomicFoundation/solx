// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A single deconstruction clause binding multiple operators (`{add as +,
// mul as *}`) on one value type. In `a + b * c`, `*` binds tighter, so the
// `mul` call nests inside the `add` call; both backends emit the inner `mul`
// call first, then the outer `add` call, in `run`.

// CHECK: sol.func @{{.*run.*}}(%{{.*}}: ui256, %{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK: sol.call @{{.*mul.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.return %{{.*}} : ui256

type T is uint256;

function add(T a, T b) pure returns (T) {
    return T.wrap(T.unwrap(a) + T.unwrap(b));
}

function mul(T a, T b) pure returns (T) {
    return T.wrap(T.unwrap(a) * T.unwrap(b));
}

using {add as +, mul as *} for T global;

contract C {
    function run(T a, T b, T c) public pure returns (T) {
        return a + b * c;
    }
}
