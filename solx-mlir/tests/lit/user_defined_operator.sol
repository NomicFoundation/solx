// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*binary_add.*}}
// CHECK:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.func @{{.*binary_and.*}}
// CHECK:   sol.call @{{.*band.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.func @{{.*binary_div.*}}
// CHECK:   sol.call @{{.*div.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.func @{{.*binary_mul.*}}
// CHECK:   sol.call @{{.*mul.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.func @{{.*binary_or.*}}
// CHECK:   sol.call @{{.*bor.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.func @{{.*binary_rem.*}}
// CHECK:   sol.call @{{.*rem.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.func @{{.*binary_sub.*}}
// CHECK:   sol.call @{{.*sub.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.func @{{.*binary_xor.*}}
// CHECK:   sol.call @{{.*bxor.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK: sol.func @{{.*precedence.*}}
// CHECK: sol.func @{{.*unary_neg.*}}
// CHECK:   sol.call @{{.*neg.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*unary_not.*}}
// CHECK:   sol.call @{{.*bnot.*}}(%{{.*}}) : (ui256) -> ui256

type T is uint256;

function add(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) + T.unwrap(b)); }

function sub(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) - T.unwrap(b)); }

function mul(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) * T.unwrap(b)); }

function div(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) / T.unwrap(b)); }

function rem(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) % T.unwrap(b)); }

function band(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) & T.unwrap(b)); }

function bor(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) | T.unwrap(b)); }

function bxor(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) ^ T.unwrap(b)); }

function bnot(T a) pure returns (T) { return T.wrap(~T.unwrap(a)); }

function neg(T a) pure returns (T) { return T.wrap(0 - T.unwrap(a)); }

using {add as +, sub as -, mul as *, div as /, rem as %, band as &, bor as |, bxor as ^, bnot as ~, neg as -} for T global;

contract C {
    function binary_add(T a, T b) public pure returns (T) { return a + b; }

    function binary_and(T a, T b) public pure returns (T) { return a & b; }

    function binary_div(T a, T b) public pure returns (T) { return a / b; }

    function binary_mul(T a, T b) public pure returns (T) { return a * b; }

    function binary_or(T a, T b) public pure returns (T) { return a | b; }

    function binary_rem(T a, T b) public pure returns (T) { return a % b; }

    function binary_sub(T a, T b) public pure returns (T) { return a - b; }

    function binary_xor(T a, T b) public pure returns (T) { return a ^ b; }

    function precedence(T a, T b, T c) public pure returns (T) { return a + b * c; }

    function unary_neg(T a) public pure returns (T) { return -a; }

    function unary_not(T a) public pure returns (T) { return ~a; }
}
