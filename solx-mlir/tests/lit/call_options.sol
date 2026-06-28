// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.bare_call %{{.*}} gas %{{.*}} value %{{.*}} input
// CHECK: sol.new "{{.*}}" value = %c0_ui256 salt = %{{.*}} ctor()
// CHECK: sol.new "{{.*}}" value = %{{[0-9]+}} salt = %{{.*}} ctor()
// CHECK: sol.ext_call "{{.*}}"({{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector

pragma solidity ^0.8.0;

interface I {
    function f(uint256 x) external payable returns (uint256);
}

contract C {
    function bare_value(address a, uint256 v, bytes memory d) external returns (bool) {
        (bool ok, ) = a.call{value: v}(d);
        return ok;
    }

    function create_salt_only(bytes32 s) external returns (Created) {
        return new Created{salt: s}();
    }

    function create_value_salt(uint256 v, bytes32 s) external returns (Created) {
        return new Created{value: v, salt: s}();
    }

    function external_value(I i, uint256 v) external returns (uint256) {
        return i.f{value: v}(7);
    }
}

contract Created {
    constructor() payable {}
}
