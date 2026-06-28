// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A ternary feeding a binary operator: the ternary materializes its result slot via
// sol.if, the join load feeds the sol.cadd. The constant-10 materialization is an
// independent op the two backends place on opposite sides of the if, so it is not
// pinned by position; the load-then-cadd after the join is the load-bearing shape.
// Symbol carries a solc node-id suffix (regex).

// CHECK-LABEL: sol.func @{{.*inbin.*}}(%{{.*}}: i1, %{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK: sol.if %{{.*}} {
// CHECK:   sol.store %{{.*}}, %[[SLOT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: } else {
// CHECK:   sol.store %{{.*}}, %[[SLOT]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: }
// CHECK: %[[L:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.cadd %[[L]], %{{.*}} : ui256

// FIX: rename the function to something nice
contract C {
    function inbin(bool c, uint256 a, uint256 b) public pure returns (uint256) {
        return (c ? a : b) + 10;
    }
}
