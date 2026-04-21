// RUN: solx --emit-mlir %s | FileCheck %s

// Short-circuit &&: result defaults to false, then-branch stores b.
// CHECK: sol.func @"logical_and(bool,bool)"
// CHECK:   %[[FALSE:.*]] = arith.constant false
// CHECK:   sol.store %[[FALSE]], %[[RES:.*]] :
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.store %{{.*}}, %[[RES]]
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:     sol.yield
// CHECK:   }
// CHECK:   %[[RET:.*]] = sol.load %[[RES]]
// CHECK:   sol.return %[[RET]]

// Short-circuit ||: result defaults to true, else-branch stores b.
// CHECK: sol.func @"logical_or(bool,bool)"
// CHECK:   %[[TRUE:.*]] = arith.constant true
// CHECK:   sol.store %[[TRUE]], %[[RES:.*]] :
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:     sol.store %{{.*}}, %[[RES]]
// CHECK:     sol.yield
// CHECK:   }
// CHECK:   %[[RET:.*]] = sol.load %[[RES]]
// CHECK:   sol.return %[[RET]]

// CHECK: sol.func @"logical_not(bool)"
// CHECK:   %[[COND:.*]] = sol.cmp eq, %{{.*}}, %{{.*}} : i1

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
