// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}(%arg0: i1) -> (ui256, ui256)
// CHECK:   %[[S0:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   %[[S1:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.if %{{.*}} {
// CHECK:     %[[G:.*]]:2 = sol.call @{{.*g.*}}() : () -> (ui256, ui256)
// CHECK:     sol.store %[[G]]#0, %[[S0]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:     sol.store %[[G]]#1, %[[S1]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:     %[[H:.*]]:2 = sol.call @{{.*h.*}}() : () -> (ui256, ui256)
// CHECK:     sol.store %[[H]]#0, %[[S0]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:     sol.store %[[H]]#1, %[[S1]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:     sol.yield
// CHECK:   }
// CHECK:   sol.return %{{.*}}, %{{.*}} : ui256, ui256

contract C {
    function g() internal pure returns (uint, uint) { return (1, 2); }

    function h() internal pure returns (uint, uint) { return (3, 4); }

    function f(bool condition) public pure returns (uint, uint) {
        return condition ? g() : h();
    }
}
