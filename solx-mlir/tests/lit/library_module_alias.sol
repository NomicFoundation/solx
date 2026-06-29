// RUN: solx --emit-mlir=sol %s | FileCheck %s

// import "..." as M module alias with an M.L.f() library call: solc's print-init aborts via
// llvm_unreachable("NYI") in genExpr(MemberAccess) at SolidityToMLIR.cpp:1698 (SIGABRT), so this is solx-only.

// CHECK: sol.func @"a()"
// CHECK:   sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK: sol.func @"g(uint256)"
// CHECK:   sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK:   sol.ext_call "f(uint256)"({{.*}}) {{.*}}library

import "library_module_alias.sol" as M;

library L {
    function f(uint256 v) external pure returns (uint256) { return v * v; }
}

contract C {
    function a() public view returns (bool) { return address(M.L) == address(0); }
    function g(uint256 v) public view returns (uint256) { return M.L.f(v); }
}
