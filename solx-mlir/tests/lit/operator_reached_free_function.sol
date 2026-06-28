// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A user-defined operator (`using {add as +}`) dispatches `x + y` to the free function `add`, whose
// body in turn calls another free function `helper`. `helper` is reachable ONLY transitively through
// the operator, so the reachability walk must descend into operator-bound function bodies (see
// `reachable_free_functions` / `walk_roots` in contract/mod.rs) — otherwise `helper` is never
// registered and emission panics with "undefined function for definition".
//
// solx-only: solc's print-init references the operator function via `sol.call` but defers free-function
// definitions to a later pass, so there is no comparable definition in its module output. The runtime
// behaviour is corpus-validated (operators/userDefined/operator_making_{view,pure}_external_call).

// FIX: this does not look good without any spacing between declarations
type T is int32;
using {add as +} for T global;
function helper(T x) pure returns (T) { return x; }
function add(T x, T y) pure returns (T) { return helper(x); }
contract C { function f(T x, T y) public pure returns (T) { return x + y; } }

// CHECK-DAG: sol.func @{{.*}}add{{.*}}(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK-DAG: sol.func @{{.*}}helper{{.*}}(%{{.*}}: si32) -> si32
// CHECK-DAG: sol.call @{{.*}}helper
