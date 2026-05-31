// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}f
// CHECK: sol.ext_icall

contract C {
    function h() public returns (uint256, uint256) {
        return (3, 4);
    }

    function f() public returns (uint256 a, uint256 b) {
        (a, b) = this.h();
    }
}
