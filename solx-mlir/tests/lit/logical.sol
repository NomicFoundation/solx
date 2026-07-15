// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

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
