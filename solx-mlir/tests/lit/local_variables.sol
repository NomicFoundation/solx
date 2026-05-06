// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Default init: alloca, store zero, load back.
// CHECK: sol.func @{{.*default_init.*}}
// CHECK:   %[[PTR:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK:   sol.store %[[ZERO]], %[[PTR]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   %[[VAL:.*]] = sol.load %[[PTR]] : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.return %[[VAL]] : ui256

// Explicit init: alloca, store initializer, load back.
// CHECK: sol.func @{{.*explicit_init.*}}
// CHECK:   %[[PTR:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.store %{{.*}}, %[[PTR]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   %[[VAL:.*]] = sol.load %[[PTR]] : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.return %[[VAL]] : ui256

// Reassign: load, compute, store back to same alloca.
// CHECK: sol.func @{{.*reassign.*}}
// CHECK:   %[[PTR:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.store %arg0, %[[PTR]]
// CHECK:   %[[V1:.*]] = sol.load %[[PTR]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]
// CHECK:   %[[V2:.*]] = sol.load %[[PTR]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]
// CHECK:   %[[RET:.*]] = sol.load %[[PTR]]
// CHECK:   sol.return %[[RET]]

contract C {
    function default_init() public pure returns (uint256) {
        uint256 x;
        return x;
    }

    function explicit_init() public pure returns (uint256) {
        uint256 x = 42;
        return x;
    }

    function reassign(uint256 x) public pure returns (uint256) {
        x = x + 1;
        x = x * 2;
        return x;
    }
}
