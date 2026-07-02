// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// internal_default's default_func_constant: solx types it to the pointer !sol.func_ref<(ui256) -> ui256>;
// solc emits the generic !sol.func_ref<() -> ()> and leans on the store slot. The two emit the functions in
// opposite order, so each backend's body is checked under its own prefix in that order.

// CHECK: sol.contract @C
// CHECK-SOLX: sol.func @"external_default()"
// CHECK-SOLX: sol.ext_func_constant {{.*}}selector = 0 {{.*}}-> !sol.ext_func_ref<() -> ()>
// CHECK-SOLX: sol.func @"internal_default()"
// CHECK-SOLX: sol.default_func_constant : !sol.func_ref<(ui256) -> ui256>
// CHECK-SOLC: sol.func @internal_default
// CHECK-SOLC: sol.default_func_constant : !sol.func_ref<() -> ()>
// CHECK-SOLC: sol.func @external_default
// CHECK-SOLC: sol.ext_func_constant {{.*}}selector = 0 {{.*}}-> !sol.ext_func_ref<() -> ()>

pragma solidity ^0.8.0;

contract C {
    function internal_default() external pure returns (uint256) {
        function(uint256) internal pure returns (uint256) f;
        return f(1);
    }

    function external_default() external view returns (address) {
        function() external x;
        return x.address;
    }
}
