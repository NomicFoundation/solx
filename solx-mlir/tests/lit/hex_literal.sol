// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*hex_val.*}}
// CHECK:   sol.constant 255 : ui8

contract C {
    function hex_val() public pure returns (uint256) {
        return 0xff;
    }
}
