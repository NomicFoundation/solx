// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A tuple-valued conditional whose BOTH branches are multi-value function calls
// (not literal tuples): the result types come from the conditional's own tuple
// type, and each branch emits a `sol.call` whose results are spread into the
// per-result stack slots. Both backends emit the identical 2-slot alloca, a
// `sol.if` with a call + 2 stores per branch, then loads + multi-return. (Only
// the internal callee symbol naming differs: solx `@"g()"`/`@"h()"`, solc
// `@g_<id>`/`@h_<id>` — regexed.)

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
    function f(bool cond) public pure returns (uint, uint) {
        return cond ? g() : h();
    }
}
