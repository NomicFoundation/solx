// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.gasleft : ui256

contract C {
    function remaining() public view returns (uint256) { return gasleft(); }
}
