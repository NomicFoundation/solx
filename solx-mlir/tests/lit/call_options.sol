// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Call options `{value: …}` / `{salt: …}` thread into the call / create op as
// explicit operands: an external call forwards the value as `msg.value`, a
// low-level `addr.call` likewise, and `new C{value, salt}` forwards the value
// and selects CREATE2 with the salt. solx-only: solx builds the external typed
// call as `sol.ext_icall` (via `sol.ext_func_constant`) where solc emits a
// symbol-callee `sol.ext_call`, so the op names differ; the value/salt operand
// wiring is identical. (The `{gas: …}` option is not yet threaded.)

// CHECK-LABEL: sol.func @{{.*}}external_value
// CHECK: sol.ext_icall {{.*}} value %{{[0-9]+}}

// CHECK-LABEL: sol.func @{{.*}}bare_value
// CHECK: sol.bare_call {{.*}} value %{{[0-9]+}} input

// CHECK-LABEL: sol.func @{{.*}}create_value_salt
// CHECK: sol.new "Created" value = %{{[0-9]+}} salt = %{{[0-9]+}} ctor

// CHECK-LABEL: sol.func @{{.*}}create_salt_only
// CHECK: sol.new "Created" value = %c0_ui256 salt = %{{[0-9]+}} ctor

pragma solidity ^0.8.0;

interface I {
    function f(uint256 x) external payable returns (uint256);
}

contract C {
    function external_value(I i, uint256 v) external returns (uint256) {
        return i.f{value: v}(7);
    }

    function bare_value(address a, uint256 v, bytes memory d) external returns (bool) {
        (bool ok, ) = a.call{value: v}(d);
        return ok;
    }

    function create_value_salt(uint256 v, bytes32 s) external returns (Created) {
        return new Created{value: v, salt: s}();
    }

    function create_salt_only(bytes32 s) external returns (Created) {
        return new Created{salt: s}();
    }
}

contract Created {
    constructor() payable {}
}
