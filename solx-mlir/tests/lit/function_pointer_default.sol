// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A default-initialised function pointer (declared without an initialiser) takes
// its type's zero value: an internal pointer is `sol.default_func_constant` (a
// pointer that reverts when called); an external one is a zero address + zero
// selector packed into an `!sol.ext_func_ref` via `sol.ext_func_constant`.

// Functions emit alphabetically by symbol, so external_default precedes
// internal_default.
// CHECK-LABEL: sol.func @{{.*}}external_default
// CHECK: sol.ext_func_constant {{.*}}selector = 0 {{.*}}-> !sol.ext_func_ref<() -> ()>

// CHECK-LABEL: sol.func @{{.*}}internal_default
// CHECK: sol.default_func_constant : !sol.func_ref<(ui256) -> ui256>

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
