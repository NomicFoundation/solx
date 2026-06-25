// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// An explicit `bytes(b)` conversion that only changes data location lowers to
// `sol.data_loc_cast` from the source location to memory.

// CHECK: sol.data_loc_cast %{{.*}} : !sol.string<CallData>, !sol.string<Memory>

contract C {
    function f(bytes calldata b) external pure returns (uint256) {
        bytes memory m = bytes(b);
        return m.length;
    }
}
