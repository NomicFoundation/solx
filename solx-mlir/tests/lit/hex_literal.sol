// RUN: slang --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*hex_val.*}}
// CHECK:   sol.constant 255 : ui8

// CHECK: sol.func @{{.*hex_wide.*}}
// CHECK:   sol.constant 4 : ui256

contract C {
    function hex_val() public pure returns (uint256) {
        return 0xff;
    }

    function hex_wide() public pure returns (uint256) {
        return 0x000000000000000000000000000000000000000000000000000000000000000004;
    }
}
