// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}inner{{.*}}(%{{.*}}: si32) -> si32

type T is int32;
using {add as +} for T global;

library M { function inner(T x) internal pure returns (T) { return x; } }

function add(T x, T y) pure returns (T) { return M.inner(x); }

contract C { function f(T x, T y) public pure returns (T) { return x + y; } }
