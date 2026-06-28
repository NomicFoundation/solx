// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*array_slice.*}}
// CHECK:   sol.slice %{{.*}}[%{{.*}} : %{{.*}}] : !sol.array<? x ui256, CallData>, ui256, ui256 -> !sol.array<? x ui256, CallData>
// CHECK: sol.func @{{.*bytes_slice.*}}
// CHECK:   sol.slice %{{.*}}[%{{.*}} : %{{.*}}] : !sol.string<CallData>, ui256, ui256 -> !sol.string<CallData>

contract C {
    function array_slice(uint256[] calldata c, uint256 s, uint256 e) external pure returns (uint256[] calldata) {
        return c[s:e];
    }

    function bytes_slice(bytes calldata b, uint256 s) external pure returns (bytes calldata) {
        return b[s:];
    }
}
