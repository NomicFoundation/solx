// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// A default-initialised function pointer (no initialiser) takes its type's zero value. For the
// external pointer both backends pack a zero address and zero selector into an !sol.ext_func_ref
// via sol.ext_func_constant. For the internal pointer solx types sol.default_func_constant to the
// variable (!sol.func_ref<(ui256) -> ui256>) while solc emits the generic !sol.func_ref<() -> ()>
// and leans on the store slot's type. The backends emit these functions in opposite order, so the
// asserts use CHECK-DAG.

// CHECK-DAG: sol.ext_func_constant {{.*}}selector = 0 {{.*}}-> !sol.ext_func_ref<() -> ()>
// CHECK-SOLX-DAG: sol.default_func_constant : !sol.func_ref<(ui256) -> ui256>
// CHECK-SOLC-DAG: sol.default_func_constant : !sol.func_ref<() -> ()>

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
