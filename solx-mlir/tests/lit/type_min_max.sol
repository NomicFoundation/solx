// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}f
// CHECK: sol.constant 255 : ui8

contract C {
    function f() public pure returns (uint8) {
        return type(uint8).max;
    }
}
