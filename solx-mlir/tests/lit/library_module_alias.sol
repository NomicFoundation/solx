// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A library qualified through a module alias: solc's print-init substitutes `sol.timestamp` for
// the address, so this is solx-only.

// CHECK: sol.func @{{.*alias_address.*}}() -> !sol.address
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   sol.return %[[ADDR]] : !sol.address

// CHECK: sol.func @{{.*alias_call.*}}
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[ARG:.*]] = sol.load
// CHECK:   sol.ext_call "{{.*visible.*}}"(%[[ARG]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c2825669559_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, static_call} : !sol.address, (ui256) -> (i1, ui256)

import "./library_module_alias.sol" as M;

library Lib {
    function visible(uint256 a) external pure returns (uint256) {
        return a;
    }
}

contract User {
    function alias_address() public pure returns (address) {
        return address(M.Lib);
    }

    function alias_call(uint256 x) public pure returns (uint256) {
        return M.Lib.visible(x);
    }
}
