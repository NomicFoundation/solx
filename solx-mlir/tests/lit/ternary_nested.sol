// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

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
