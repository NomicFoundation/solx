// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// Short-circuit &&: alloca a result slot, seed it, conditionally store b in
// the then-branch, then load and return. Both compilers seed with
// `arith.constant false`.
// CHECK: sol.func @{{.*logical_and.*}}
// CHECK:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   %[[RES:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   sol.store %{{.*}}, %[[RES]]
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.store %{{.*}}, %[[RES]]
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:   }
// CHECK:   sol.load %[[RES]]
// CHECK:   sol.return

// Short-circuit ||: same shape, with the conditional store in the else-branch.
// Seed differs: solc stores the loaded `a` (a SSA register); solx stores
// `arith.constant true` (constant-folded "if a is true the result is true").
// CHECK: sol.func @{{.*logical_or.*}}
// CHECK:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   %[[RES:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLC:   sol.store %{{.*}}, %[[RES]]
// CHECK-SOLX:   %[[T:.*]] = arith.constant true
// CHECK-SOLX:   sol.store %[[T]], %[[RES]]
// CHECK:   sol.if %{{.*}} {
// CHECK:   } else {
// CHECK:     sol.store %{{.*}}, %[[RES]]
// CHECK:     sol.yield
// CHECK:   }
// CHECK:   sol.load %[[RES]]
// CHECK:   sol.return

// CHECK: sol.func @{{.*logical_not.*}}
// CHECK:   sol.cmp eq, %{{.*}}, %{{.*}} : i1

contract C {
    function logical_and(bool a, bool b) public pure returns (bool) {
        return a && b;
    }

    function logical_or(bool a, bool b) public pure returns (bool) {
        return a || b;
    }

    function logical_not(bool a) public pure returns (bool) {
        return !a;
    }
}
