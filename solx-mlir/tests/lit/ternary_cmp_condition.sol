// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-LABEL: sol.func @{{.*tcmpcond.*}}(%{{.*}}: ui256, %{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK: %[[C:.*]] = sol.cmp gt, %{{.*}}, %{{.*}} : ui256
// CHECK: sol.if %[[C]] {
// CHECK:   sol.store %{{.*}}, %[[SLOT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: } else {
// CHECK:   sol.store %{{.*}}, %[[SLOT]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: }
// CHECK: sol.load %[[SLOT]] : !sol.ptr<ui256, Stack>, ui256

contract C {
    function tcmpcond(uint256 x, uint256 a, uint256 b) public pure returns (uint256) {
        return x > 5 ? a : b;
    }
}
