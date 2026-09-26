// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.sig : !sol.fixedbytes<4>

contract C {
    function selector() public pure returns (bytes4) { return msg.sig; }
}
