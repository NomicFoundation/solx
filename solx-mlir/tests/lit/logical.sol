// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Short-circuit &&: alloca a result slot, seed it, conditionally store b in
// the then-branch, then load and return. solx and solc seed the slot
// differently (constant-false vs the loaded `a`), so we don't pin the seed.
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

// Short-circuit ||: same shape, the conditional store lands in the else-branch.
// CHECK: sol.func @{{.*logical_or.*}}
// CHECK:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   %[[RES:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   sol.store %{{.*}}, %[[RES]]
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
