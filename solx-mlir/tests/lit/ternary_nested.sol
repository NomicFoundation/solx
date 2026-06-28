// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A nested ternary lowers to nested sol.if regions, each writing a stack slot read
// after the if. The outer slot is written by the inner ternary's loaded result in
// the then-branch and by `d` in the else-branch. Both backends emit byte-identical
// op structure; the symbol name carries a solc node-id suffix (regex). One function
// keeps the CHECK-LABEL order stable across the alphabetical/source-order walk.

// CHECK-LABEL: sol.func @{{.*nested.*}}(%{{.*}}: i1, %{{.*}}: i1, %{{.*}}: ui256, %{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK: sol.if %{{.*}} {
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.store %{{.*}}, %[[INNER:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:     sol.store %{{.*}}, %[[INNER]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:     sol.yield
// CHECK:   }
// CHECK:   sol.load %[[INNER]] : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.store %{{.*}}, %[[OUTER:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: } else {
// CHECK:   sol.store %{{.*}}, %[[OUTER]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: }
// CHECK: sol.load %[[OUTER]] : !sol.ptr<ui256, Stack>, ui256

contract C {
    function nested(bool c1, bool c2, uint256 a, uint256 b, uint256 d) public pure returns (uint256) {
        return c1 ? (c2 ? a : b) : d;
    }
}
