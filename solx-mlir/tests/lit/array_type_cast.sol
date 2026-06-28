// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[V:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.array<? x ui256, CallData>, Stack>, !sol.array<? x ui256, CallData>
// CHECK: sol.data_loc_cast %[[V]] : !sol.array<? x ui256, CallData>, !sol.array<? x ui256, Memory>

contract C {
    function f(uint256[] calldata data) external pure returns (uint256[] memory) {
        return uint256[](data);
    }
}
