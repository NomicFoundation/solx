// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*add.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256

// CHECK: sol.func @{{.*plus.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*sum.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*f.*}}() -> ui256
// CHECK:   sol.call @{{.*f.*}}() : () -> ui256

type T is uint256;

using {add as +} for T global;

function add(T a, T b) pure returns (T) {
    return T.wrap(T.unwrap(a) + T.unwrap(b));
}

function f() pure returns (uint256) {
    return 1337;
}

contract C {
    function add(T a, T b) internal pure returns (T) {
        return T.wrap(T.unwrap(a) + T.unwrap(b) + 1);
    }

    function plus(T a, T b) public pure returns (T) {
        return a + b;
    }

    function sum(T a, T b) public pure returns (T) {
        return add(a, b);
    }

    function f() public pure returns (uint256) {
        return f();
    }
}
