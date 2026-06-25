// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A user-defined operator (`using {add as +}`) dispatches `x + y` to the free function `add`, whose
// body calls an INTERNAL library function `M.inner()`. `inner` is reachable only transitively through
// the operator, so BOTH reachability walks must descend into operator-bound bodies — not just the
// free-function walk (`reachable_free_functions`) but also the library walk
// (`reachable_library_functions`, seeded with the operator roots in contract/mod.rs). Otherwise `inner`
// is never registered and emission panics with "undefined function for definition". solc emits the
// internal library function as a plain sol.func, so this is shared parity.

type T is int32;
using {add as +} for T global;
library M { function inner(T x) internal pure returns (T) { return x; } }
function add(T x, T y) pure returns (T) { return M.inner(x); }
contract C { function f(T x, T y) public pure returns (T) { return x + y; } }

// CHECK: sol.func @{{.*}}inner{{.*}}(%{{.*}}: si32) -> si32
