// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*read.*}}() -> ui256
// CHECK:   sol.alloca : !sol.ptr<!sol.array<6 x ui256, Memory>, Stack>
// CHECK:   sol.malloc zero_init :  !sol.array<6 x ui256, Memory>
// CHECK:   %[[A:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.array<6 x ui256, Memory>, Stack>, !sol.array<6 x ui256, Memory>
// CHECK:   %[[L:.*]] = sol.length %[[A]] : !sol.array<6 x ui256, Memory>
// CHECK:   sol.return %[[L]] : ui256

contract C {
    uint256 constant N = 3;

    function read() public pure returns (uint256) {
        uint256[N * 2] memory array;
        return array.length;
    }
}
