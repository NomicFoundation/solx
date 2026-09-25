// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.get_calldata : !sol.string<CallData>

contract C {
    function payload() external view returns (bytes calldata) { return msg.data; }
}
