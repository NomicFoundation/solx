// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}f
// CHECK: sol.address_cast %{{.*}} : !sol.address to !sol.contract<"I">
// CHECK: sol.ext_icall

interface I {
    function g(uint256) external returns (uint256);
}

contract C {
    function f(address a) public returns (uint256) {
        return I(a).g(7);
    }
}
