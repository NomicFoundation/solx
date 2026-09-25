// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*blank.*}}
// CHECK: sol.cast %c7_ui8

contract C {
    function blank() public pure returns (uint256) {
        uint256 a;
        (a, ) = (7, 8);
        return a;
    }
}
