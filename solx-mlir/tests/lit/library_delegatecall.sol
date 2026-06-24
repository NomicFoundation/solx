// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A direct external/public library call (`L.f(args)`) is a delegatecall to the
// deployed library: the native `sol.ext_call` carries the `delegate_call` +
// `library_call` flags, the library address from `sol.lib_addr`, the function
// selector, and the callee type — its lowering owns the ABI encode, the
// delegatecall, the revert-bubble, and the result decode. solx-only: the callee
// label differs from solc's (the call dispatches on selector + address, not the
// label).

// CHECK-LABEL: sol.func @{{.*}}g
// CHECK: sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK: sol.ext_call {{.*}}selector {{.*}}{{{.*}}delegate_call{{.*}}library_call{{.*}}}

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
