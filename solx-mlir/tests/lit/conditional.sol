// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*ternary.*}}(%{{.*}}: i1, %{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.store %{{.*}}, %[[SLOT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   } else {
// CHECK:     sol.store %{{.*}}, %[[SLOT]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   }
// CHECK:   sol.load %[[SLOT]] : !sol.ptr<ui256, Stack>, ui256

contract C {
    function ternary(bool c, uint256 a, uint256 b) public pure returns (uint256) {
        return c ? a : b;
    }
}
