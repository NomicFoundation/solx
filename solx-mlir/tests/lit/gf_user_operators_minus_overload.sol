// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// `-` bound BOTH as binary subtraction and unary negation on the same value
// type; the using-operator mapping disambiguates by arity (arity 2 -> Sub,
// arity 1 -> Neg). Each `-` use dispatches to the matching bound function.
// Function emission order diverges: solx walks alphabetically (doneg, dosub),
// solc in source order (dosub, doneg), so split prefixes pin each side.

// CHECK-SOLX: sol.func @{{.*doneg.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX: sol.call @{{.*neg.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK-SOLX: sol.func @{{.*dosub.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK-SOLX: sol.call @{{.*sub.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK-SOLC: sol.func @{{.*dosub.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK-SOLC: sol.call @{{.*sub.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK-SOLC: sol.func @{{.*doneg.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC: sol.call @{{.*neg.*}}(%{{.*}}) : (ui256) -> ui256

type T is uint256;

function sub(T a, T b) pure returns (T) {
    return T.wrap(T.unwrap(a) - T.unwrap(b));
}

function neg(T a) pure returns (T) {
    return T.wrap(0 - T.unwrap(a));
}

using {sub as -, neg as -} for T global;

contract C {
    function dosub(T a, T b) public pure returns (T) {
        return a - b;
    }

    function doneg(T a) public pure returns (T) {
        return -a;
    }
}
