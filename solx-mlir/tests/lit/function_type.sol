// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*callExternal.*}}(%{{.*}}: !sol.ext_func_ref<(ui256) -> ui256>) -> ui256
// CHECK:   sol.ext_icall %{{.*}} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)
// CHECK: sol.func @{{.*callInternal.*}}() -> ui256
// CHECK:   sol.func_constant @{{.*}} : !sol.func_ref<(ui256) -> ui256>
// CHECK:   sol.icall %{{.*}} : !sol.func_ref<(ui256) -> ui256>, (ui256) -> ui256

contract C {
    function callExternal(function(uint256) external returns (uint256) fp) public returns (uint256) {
        return fp(3);
    }

    function callInternal() public pure returns (uint256) {
        function(uint256) internal pure returns (uint256) fp = g;
        return fp(7);
    }

    function g(uint256 x) internal pure returns (uint256) { return x; }
}
