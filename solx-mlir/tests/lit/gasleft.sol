// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.gasleft : ui256

contract C {
    function remaining() public view returns (uint256) { return gasleft(); }
}
