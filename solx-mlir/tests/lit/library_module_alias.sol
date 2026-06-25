// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A library reached through an aliased import (`import "self" as M; ... M.L`) is resolved
// transparently to the library: `address(M.L)` emits the same `sol.lib_addr` link placeholder, and
// `M.L.f(v)` the same `sol.ext_call` (delegatecall + library_call) as a direct `L.f(v)`. solx-only:
// solc's MLIR frontend does not resolve member access through an import alias (NYI), so the alias is
// a solx capability emitting the canonical library ops the shared backend already lowers.

import "library_module_alias.sol" as M;

library L {
    function f(uint256 v) external pure returns (uint256) { return v * v; }
}

contract C {
    function a() public view returns (bool) { return address(M.L) == address(0); }
    function g(uint256 v) public view returns (uint256) { return M.L.f(v); }
}

// CHECK: sol.func @"a()"
// CHECK:   sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK: sol.func @"g(uint256)"
// CHECK:   sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK:   sol.ext_call "f(uint256)"({{.*}}) {{.*}}library
