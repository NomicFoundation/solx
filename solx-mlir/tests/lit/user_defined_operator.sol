// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s
// solc's print-init emits the contract alone, so the CHECKs stop at the call sites.

// CHECK: sol.func @{{.*plus.*}}-> ui256
// CHECK:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*minus.*}}-> ui256
// CHECK:   sol.call @{{.*sub.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*times.*}}-> ui256
// CHECK:   sol.call @{{.*mul.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*divided.*}}-> ui256
// CHECK:   sol.call @{{.*div.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*modulo.*}}-> ui256
// CHECK:   sol.call @{{.*rem.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*conjunction.*}}-> ui256
// CHECK:   sol.call @{{.*band.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*disjunction.*}}-> ui256
// CHECK:   sol.call @{{.*bor.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*exclusive.*}}-> ui256
// CHECK:   sol.call @{{.*bxor.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*negated.*}}-> ui256
// CHECK:   sol.call @{{.*neg.*}}(%{{.*}}) : (ui256) -> ui256

// CHECK: sol.func @{{.*inverted.*}}-> ui256
// CHECK:   sol.call @{{.*bnot.*}}(%{{.*}}) : (ui256) -> ui256

// CHECK: sol.func @{{.*equal.*}}-> i1
// CHECK:   sol.call @{{.*eq.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> i1

// CHECK: sol.func @{{.*unequal.*}}-> i1
// CHECK:   sol.call @{{.*ne.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> i1

// CHECK: sol.func @{{.*below.*}}-> i1
// CHECK:   sol.call @{{.*lt.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> i1

// CHECK: sol.func @{{.*at_most.*}}-> i1
// CHECK:   sol.call @{{.*le.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> i1

// CHECK: sol.func @{{.*above.*}}-> i1
// CHECK:   sol.call @{{.*gt.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> i1

// CHECK: sol.func @{{.*at_least.*}}-> i1
// CHECK:   sol.call @{{.*ge.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> i1

// CHECK: sol.func @{{.*precedence.*}}-> ui256
// CHECK:   sol.call @{{.*mul.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
// CHECK:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*signed_equal.*}}-> i1
// CHECK:   sol.call @{{.*signed_eq.*}}(%{{.*}}, %{{.*}}) : (si256, si256) -> i1

// CHECK: sol.func @{{.*signed_below.*}}-> i1
// CHECK:   sol.call @{{.*signed_lt.*}}(%{{.*}}, %{{.*}}) : (si256, si256) -> i1

type T is uint256;
type Signed is int256;

using {
    add as +, sub as -, mul as *, div as /, rem as %,
    band as &, bor as |, bxor as ^,
    neg as -, bnot as ~,
    eq as ==, ne as !=, lt as <, le as <=, gt as >, ge as >=
} for T global;

using {signed_eq as ==, signed_lt as <} for Signed global;

function add(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) + T.unwrap(b)); }
function sub(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) - T.unwrap(b)); }
function mul(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) * T.unwrap(b)); }
function div(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) / T.unwrap(b)); }
function rem(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) % T.unwrap(b)); }
function band(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) & T.unwrap(b)); }
function bor(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) | T.unwrap(b)); }
function bxor(T a, T b) pure returns (T) { return T.wrap(T.unwrap(a) ^ T.unwrap(b)); }
function neg(T a) pure returns (T) { return T.wrap(0 - T.unwrap(a)); }
function bnot(T a) pure returns (T) { return T.wrap(~T.unwrap(a)); }
function eq(T a, T b) pure returns (bool) { return T.unwrap(a) == T.unwrap(b); }
function ne(T a, T b) pure returns (bool) { return T.unwrap(a) != T.unwrap(b); }
function lt(T a, T b) pure returns (bool) { return T.unwrap(a) < T.unwrap(b); }
function le(T a, T b) pure returns (bool) { return T.unwrap(a) <= T.unwrap(b); }
function gt(T a, T b) pure returns (bool) { return T.unwrap(a) > T.unwrap(b); }
function ge(T a, T b) pure returns (bool) { return T.unwrap(a) >= T.unwrap(b); }

function signed_eq(Signed a, Signed b) pure returns (bool) { return Signed.unwrap(a) == Signed.unwrap(b); }
function signed_lt(Signed a, Signed b) pure returns (bool) { return Signed.unwrap(a) < Signed.unwrap(b); }

contract C {
    function plus(T a, T b) public pure returns (T) { return a + b; }
    function minus(T a, T b) public pure returns (T) { return a - b; }
    function times(T a, T b) public pure returns (T) { return a * b; }
    function divided(T a, T b) public pure returns (T) { return a / b; }
    function modulo(T a, T b) public pure returns (T) { return a % b; }
    function conjunction(T a, T b) public pure returns (T) { return a & b; }
    function disjunction(T a, T b) public pure returns (T) { return a | b; }
    function exclusive(T a, T b) public pure returns (T) { return a ^ b; }
    function negated(T a) public pure returns (T) { return -a; }
    function inverted(T a) public pure returns (T) { return ~a; }
    function equal(T a, T b) public pure returns (bool) { return a == b; }
    function unequal(T a, T b) public pure returns (bool) { return a != b; }
    function below(T a, T b) public pure returns (bool) { return a < b; }
    function at_most(T a, T b) public pure returns (bool) { return a <= b; }
    function above(T a, T b) public pure returns (bool) { return a > b; }
    function at_least(T a, T b) public pure returns (bool) { return a >= b; }
    function precedence(T a, T b, T c) public pure returns (T) { return a + b * c; }
    function signed_equal(Signed a, Signed b) public pure returns (bool) { return a == b; }
    function signed_below(Signed a, Signed b) public pure returns (bool) { return a < b; }
}
