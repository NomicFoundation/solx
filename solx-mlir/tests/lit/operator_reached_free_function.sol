// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Free function reached only via a user-defined operator binding (using {add as +}): solc's
// print-init compiles but omits the add and helper free functions, calling @add yet never lowering it, so this is solx-only.
//
// A reached free function is emitted under its signature suffixed with `_<node-id>`, the bare integer
// that disambiguates two like-signatured free functions, and every call site agrees with its definition.

// CHECK: sol.func @{{.*}}f{{.*}}(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK:   sol.call @{{.*}}add{{.*}}_[[ADD:[0-9]+]]"(%{{.*}}, %{{.*}}) : (si32, si32) -> si32
// CHECK:   sol.return %{{.*}} : si32
// CHECK: sol.func @{{.*}}helper{{.*}}_[[HELPER:[0-9]+]]"(%{{.*}}: si32) -> si32
// CHECK: sol.func @{{.*}}add{{.*}}_[[ADD]]"(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK:   sol.call @{{.*}}helper{{.*}}_[[HELPER]]"

type T is int32;
using {add as +} for T global;

function helper(T x) pure returns (T) { return x; }

function add(T x, T y) pure returns (T) { return helper(x); }

contract C { function f(T x, T y) public pure returns (T) { return x + y; } }
