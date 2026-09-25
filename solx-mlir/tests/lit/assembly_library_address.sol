// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*target.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[ADDR:.*]] = sol.lib_addr "{{.*}}L" : !sol.address
// CHECK:     sol.yul_val_cast %[[ADDR]] : !sol.address -> i256

library L {
    function g() internal pure returns (uint256) {
        return 1;
    }
}

contract C {
    function target() public pure returns (uint256 r) {
        assembly {
            r := L
        }
    }
}
