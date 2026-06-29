// RUN: solx --emit-mlir=sol %s | FileCheck %s

// using-for binding an external library function (lowered to a delegatecall library_call):
// solc's print-init hits NYI and aborts at SolidityToMLIR.cpp:1698, so this is solx-only.

// CHECK-LABEL: sol.func @{{.*}}f
// CHECK: %[[X:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK: sol.ext_call "add(uint256,uint256)"(%[[X]], %{{.*}}){{.*}}{{{.*}}delegate_call{{.*}}library_call{{.*}}}

library L {
    function add(uint256 a, uint256 b) external returns (uint256) {
        return a + b;
    }
}

contract C {
    using L for uint256;

    function f(uint256 x) external returns (uint256) {
        return x.add(3);
    }
}
