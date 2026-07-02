// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-LABEL: sol.func @{{.*}}g
// CHECK: sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK: sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, delegate_call, library_call}

pragma solidity ^0.8.0;

library L {
    function f(uint256 x) external returns (uint256) {
        return x + 1;
    }
}

contract C {
    function g() external returns (uint256) {
        return L.f(7);
    }
}
