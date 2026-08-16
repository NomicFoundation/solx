// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc rejects call options on library delegatecalls ("Function call options can only be set on
// external function calls or contract creations"); slang admits them and solx honors the named
// gas, so this is solx-only.

// CHECK: sol.func @{{.*qualified_gas.*}}
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[GAS:.*]] = sol.load
// CHECK:   %[[ARG:.*]] = sol.load
// CHECK:   sol.ext_call "{{.*mutate.*}}"(%[[ARG]]) at %[[ADDR]] gas %[[GAS]] value %c0_ui256 selector %c1899731083_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, static_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*attached_gas.*}}
// CHECK:   %[[GAS:.*]] = sol.load
// CHECK:   %[[ARG:.*]] = sol.load
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   sol.ext_call "{{.*mutate.*}}"(%[[ARG]]) at %[[ADDR]] gas %[[GAS]] value %c0_ui256 selector %c1899731083_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, static_call} : !sol.address, (ui256) -> (i1, ui256)

library Lib {
    function mutate(uint256 a) external pure returns (uint256) {
        return a + 1;
    }
}

contract User {
    using Lib for uint256;

    function qualified_gas(uint256 x, uint256 g) public view returns (uint256) {
        return Lib.mutate{gas: g}(x);
    }

    function attached_gas(uint256 x, uint256 g) public view returns (uint256) {
        return x.mutate{gas: g}();
    }
}
