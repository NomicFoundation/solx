// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"hex_val()"
// CHECK:   sol.constant 255 : ui8

contract C {
    function hex_val() public pure returns (uint256) {
        return 0xff;
    }
}
