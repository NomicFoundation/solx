// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// A user-defined operator (`using {add as +}`) binds `x + y` to the free function `add`, which in turn
// calls `helper`, reachable only transitively through the operator. Both backends emit `f`'s body and
// its `sol.call` to `add`. solx
// additionally emits the free-function definitions (`sol.func` for `add`/`helper`, their stack slots,
// and the `sol.call @helper` inside `add`'s body); solc omits those bodies. CHECK-SOLX pins the
// solx-only definitions; CHECK-SOLC asserts solc omits them.

// CHECK: sol.contract @C
// CHECK: sol.func @{{.*}}f{{.*}}(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK: sol.call @{{.*}}add{{.*}}(%{{.*}}, %{{.*}}) : (si32, si32) -> si32
// CHECK: sol.return %{{.*}} : si32
// CHECK-SOLX: sol.func @{{.*}}helper{{.*}}(%{{.*}}: si32) -> si32
// CHECK-SOLX: sol.func @{{.*}}add{{.*}}(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK-SOLX: sol.call @{{.*}}helper
// CHECK-SOLC-NOT: sol.func @{{.*}}helper
// CHECK-SOLC-NOT: sol.func @{{.*}}add

type T is int32;
using {add as +} for T global;
function helper(T x) pure returns (T) { return x; }
function add(T x, T y) pure returns (T) { return helper(x); }
contract C { function f(T x, T y) public pure returns (T) { return x + y; } }
