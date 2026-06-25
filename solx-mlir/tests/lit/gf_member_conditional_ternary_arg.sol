// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A ternary used as a call argument is materialized into a stack slot (via sol.if),
// loaded, then passed to the call. Both backends emit identical structure; the
// callee symbol carries a solc node-id suffix (regex). Only the `asarg` function
// carries the ternary; the internal `id` callee has no body op we pin on, so a
// single CHECK-LABEL block keeps ordering stable across the two walk orders.

// CHECK-LABEL: sol.func @{{.*asarg.*}}(%{{.*}}: i1, %{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK: sol.if %{{.*}} {
// CHECK:   sol.store %{{.*}}, %[[SLOT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: } else {
// CHECK:   sol.store %{{.*}}, %[[SLOT]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: }
// CHECK: %[[ARG:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.call @{{.*id.*}}(%[[ARG]]) : (ui256) -> ui256

contract C {
    function id(uint256 v) internal pure returns (uint256) { return v; }
    function asarg(bool c, uint256 a, uint256 b) public pure returns (uint256) {
        return id(c ? a : b);
    }
}
