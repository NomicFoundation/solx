// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// Call options thread `{value: ...}` / `{salt: ...}` into the call / create op as explicit operands.
// An external call and a low-level `addr.call` forward the value as msg.value; `new C{value, salt}`
// forwards the value and selects CREATE2 via the salt. The value and salt operand wiring is identical
// between the backends. Only the external typed call differs: solx emits `sol.ext_func_constant` plus
// `sol.ext_icall` over an ext_func_ref callee, while solc emits a symbol-callee `sol.ext_call` carrying
// an explicit selector operand. CHECK-SOLX pins the solx form and CHECK-SOLC the solc form. The two
// backends emit the functions in different orders, so the block matches order-independently.

// CHECK-SOLX-DAG: sol.ext_func_constant %{{.*}} {selector = {{.*}}} : !sol.address -> !sol.ext_func_ref
// CHECK-SOLX-DAG: sol.ext_icall %{{.*}} gas %{{.*}} value %{{.*}}
// CHECK-SOLC-DAG: sol.ext_call "{{.*}}"({{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector
// CHECK-DAG: sol.bare_call %{{.*}} gas %{{.*}} value %{{.*}} input
// CHECK-DAG: sol.new "{{.*}}" value = %{{[0-9]+}} salt = %{{.*}} ctor()
// CHECK-DAG: sol.new "{{.*}}" value = %c0_ui256 salt = %{{.*}} ctor()

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
