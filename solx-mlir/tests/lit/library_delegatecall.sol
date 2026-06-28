// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// A direct external library call (`L.f(args)`) lowers to a `sol.ext_call` that carries the
// `delegate_call` and `library_call` flags, the `sol.lib_addr` library address, the selector, and
// the callee type; that one op owns the ABI encode, the delegatecall, the revert bubble, and the
// result decode. The callee label diverges: solx emits the full signature `f(uint256)`, solc the
// mangled `f_<id>`. CHECK-SOLX pins the solx label, CHECK-SOLC the solc one; the call dispatches on
// selector and address, not the label, so the difference is benign.

// CHECK-LABEL: sol.func @{{.*}}g
// CHECK: sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK-SOLX: sol.ext_call "f(uint256)"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, delegate_call, library_call}
// CHECK-SOLC: sol.ext_call "f_{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, delegate_call, library_call}

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
