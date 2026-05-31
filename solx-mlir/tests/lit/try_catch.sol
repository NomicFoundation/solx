// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}f
// CHECK: sol.ext_icall {{.*}}{try_call}
// CHECK: sol.if

interface I {
    function g() external returns (uint256);
}

contract C {
    function f(address a) public returns (uint256 r) {
        try I(a).g() returns (uint256 v) {
            r = v;
        } catch {
            r = 99;
        }
    }
}
