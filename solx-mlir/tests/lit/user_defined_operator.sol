// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A user-defined operator bound via `using {f as OP} for T global` dispatches the
// operation to the bound function as a `sol.call`, never the native op. `-` is bound
// to both binary `sub` and unary `neg`, disambiguated by arity; `~` and unary `-` are
// the unary forms. The bound symbol carries a NodeId suffix, matched loosely.

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
    // CHECK-DAG: sol.func @{{.*binary_add.*}}
    // CHECK-DAG:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
    function binary_add(T a, T b) public pure returns (T) { return a + b; }

    // CHECK-DAG: sol.func @{{.*binary_sub.*}}
    // CHECK-DAG:   sol.call @{{.*sub.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
    function binary_sub(T a, T b) public pure returns (T) { return a - b; }

    // CHECK-DAG: sol.func @{{.*binary_mul.*}}
    // CHECK-DAG:   sol.call @{{.*mul.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
    function binary_mul(T a, T b) public pure returns (T) { return a * b; }

    // CHECK-DAG: sol.func @{{.*binary_div.*}}
    // CHECK-DAG:   sol.call @{{.*div.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
    function binary_div(T a, T b) public pure returns (T) { return a / b; }

    // CHECK-DAG: sol.func @{{.*binary_rem.*}}
    // CHECK-DAG:   sol.call @{{.*rem.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
    function binary_rem(T a, T b) public pure returns (T) { return a % b; }

    // CHECK-DAG: sol.func @{{.*binary_and.*}}
    // CHECK-DAG:   sol.call @{{.*band.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
    function binary_and(T a, T b) public pure returns (T) { return a & b; }

    // CHECK-DAG: sol.func @{{.*binary_or.*}}
    // CHECK-DAG:   sol.call @{{.*bor.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
    function binary_or(T a, T b) public pure returns (T) { return a | b; }

    // CHECK-DAG: sol.func @{{.*binary_xor.*}}
    // CHECK-DAG:   sol.call @{{.*bxor.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256
    function binary_xor(T a, T b) public pure returns (T) { return a ^ b; }

    // CHECK-DAG: sol.func @{{.*unary_not.*}}
    // CHECK-DAG:   sol.call @{{.*bnot.*}}(%{{.*}}) : (ui256) -> ui256
    function unary_not(T a) public pure returns (T) { return ~a; }

    // CHECK-DAG: sol.func @{{.*unary_neg.*}}
    // CHECK-DAG:   sol.call @{{.*neg.*}}(%{{.*}}) : (ui256) -> ui256
    function unary_neg(T a) public pure returns (T) { return -a; }

    // A single using clause binds many operators; `*` binds tighter than `+`, so the
    // `mul` call nests inside the `add` call.
    // CHECK-DAG: sol.func @{{.*precedence.*}}
    function precedence(T a, T b, T c) public pure returns (T) { return a + b * c; }
}
