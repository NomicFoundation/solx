// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}(%arg0: i1, %arg1: i1) -> (ui256, ui256)
// CHECK:   %[[O0:.*]] = sol.alloca : !sol.ptr<ui8, Stack>
// CHECK:   %[[O1:.*]] = sol.alloca : !sol.ptr<ui8, Stack>
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.store %{{.*}}, %[[O0]] : ui8, !sol.ptr<ui8, Stack>
// CHECK:     sol.store %{{.*}}, %[[O1]] : ui8, !sol.ptr<ui8, Stack>
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:     %[[I0:.*]] = sol.alloca : !sol.ptr<ui8, Stack>
// CHECK:     %[[I1:.*]] = sol.alloca : !sol.ptr<ui8, Stack>
// CHECK:     sol.if %{{.*}} {
// CHECK:       sol.store %{{.*}}, %[[I0]] : ui8, !sol.ptr<ui8, Stack>
// CHECK:       sol.store %{{.*}}, %[[I1]] : ui8, !sol.ptr<ui8, Stack>
// CHECK:       sol.yield
// CHECK:     } else {
// CHECK:       sol.store %{{.*}}, %[[I0]] : ui8, !sol.ptr<ui8, Stack>
// CHECK:       sol.store %{{.*}}, %[[I1]] : ui8, !sol.ptr<ui8, Stack>
// CHECK:       sol.yield
// CHECK:     }
// CHECK:   sol.return %{{.*}}, %{{.*}} : ui256, ui256

contract C {
    function f(bool a, bool b) public pure returns (uint, uint) {
        return a ? (1, 2) : b ? (3, 4) : (5, 6);
    }
}
