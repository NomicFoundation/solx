// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Short-circuit &&: alloca a result slot, seed it, conditionally store b in the
// then-branch, then load and return (both compilers seed with `sol.constant
// false`). ||: same shape with the conditional store in the else-branch, and the
// seed differs - solc stores the loaded `a`, solx stores `sol.constant true`.
// solx walks functions alphabetically and solc in source order, so each
// backend's CHECK sequence follows its own function order.

// CHECK-SOLX: sol.func @{{.*logical_and.*}}
// CHECK-SOLX:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLX:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLX:   %[[RES:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLX:   sol.store %{{.*}}, %[[RES]]
// CHECK-SOLX:   sol.if %{{.*}} {
// CHECK-SOLX:     sol.store %{{.*}}, %[[RES]]
// CHECK-SOLX:     sol.yield
// CHECK-SOLX:   } else {
// CHECK-SOLX:   }
// CHECK-SOLX:   sol.load %[[RES]]
// CHECK-SOLX:   sol.return
// CHECK-SOLX: sol.func @{{.*logical_not.*}}
// CHECK-SOLX:   sol.cmp eq, %{{.*}}, %{{.*}} : i1
// CHECK-SOLX: sol.func @{{.*logical_or.*}}
// CHECK-SOLX:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLX:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLX:   %[[RES:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLX:   %[[T:.*]] = sol.constant true
// CHECK-SOLX:   sol.store %[[T]], %[[RES]]
// CHECK-SOLX:   sol.if %{{.*}} {
// CHECK-SOLX:   } else {
// CHECK-SOLX:     sol.store %{{.*}}, %[[RES]]
// CHECK-SOLX:     sol.yield
// CHECK-SOLX:   }
// CHECK-SOLX:   sol.load %[[RES]]
// CHECK-SOLX:   sol.return

// CHECK-SOLC: sol.func @{{.*logical_and.*}}
// CHECK-SOLC:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLC:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLC:   %[[RES:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLC:   sol.store %{{.*}}, %[[RES]]
// CHECK-SOLC:   sol.if %{{.*}} {
// CHECK-SOLC:     sol.store %{{.*}}, %[[RES]]
// CHECK-SOLC:     sol.yield
// CHECK-SOLC:   } else {
// CHECK-SOLC:   }
// CHECK-SOLC:   sol.load %[[RES]]
// CHECK-SOLC:   sol.return
// CHECK-SOLC: sol.func @{{.*logical_or.*}}
// CHECK-SOLC:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLC:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLC:   %[[RES:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK-SOLC:   sol.store %{{.*}}, %[[RES]]
// CHECK-SOLC:   sol.if %{{.*}} {
// CHECK-SOLC:   } else {
// CHECK-SOLC:     sol.store %{{.*}}, %[[RES]]
// CHECK-SOLC:     sol.yield
// CHECK-SOLC:   }
// CHECK-SOLC:   sol.load %[[RES]]
// CHECK-SOLC:   sol.return
// CHECK-SOLC: sol.func @{{.*logical_not.*}}
// CHECK-SOLC:   sol.cmp eq, %{{.*}}, %{{.*}} : i1

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
