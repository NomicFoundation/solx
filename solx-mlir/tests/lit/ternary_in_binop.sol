// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-LABEL: sol.func @{{.*ternary_in_addition.*}}(%{{.*}}: i1, %{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK: sol.if %{{.*}} {
// CHECK:   sol.store %{{.*}}, %[[SLOT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: } else {
// CHECK:   sol.store %{{.*}}, %[[SLOT]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: }
// CHECK: %[[L:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.cadd %[[L]], %{{.*}} : ui256

contract C {
    function ternary_in_addition(bool c, uint256 a, uint256 b) public pure returns (uint256) {
        return (c ? a : b) + 10;
    }
}
