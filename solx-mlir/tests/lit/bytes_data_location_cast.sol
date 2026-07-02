// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.data_loc_cast %{{.*}} : !sol.string<CallData>, !sol.string<Memory>

contract C {
    function f(bytes calldata b) external pure returns (uint256) {
        bytes memory m = bytes(b);
        return m.length;
    }
}
