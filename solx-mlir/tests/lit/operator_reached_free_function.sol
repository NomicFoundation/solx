// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Free function reached only via a user-defined operator binding (using {add as +}): solc's
// print-init compiles but omits the add and helper free functions, calling @add yet never lowering it, so this is solx-only.

// CHECK: sol.func @{{.*}}f{{.*}}(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK:   sol.call @{{.*}}add{{.*}}(%{{.*}}, %{{.*}}) : (si32, si32) -> si32
// CHECK:   sol.return %{{.*}} : si32
// CHECK: sol.func @{{.*}}helper{{.*}}(%{{.*}}: si32) -> si32
// CHECK: sol.func @{{.*}}add{{.*}}(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK:   sol.call @{{.*}}helper

type T is int32;
using {add as +} for T global;

function helper(T x) pure returns (T) { return x; }

function add(T x, T y) pure returns (T) { return helper(x); }

contract C { function f(T x, T y) public pure returns (T) { return x + y; } }
