// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Function-type variables exercise the `Function` arm of type resolution: an
// internal function pointer resolves to `!sol.func_ref<...>` (stored on the
// stack, invoked via `sol.icall`); an external one resolves to
// `!sol.ext_func_ref<...>` (invoked via `sol.ext_icall`). Both backends emit
// identical types and call ops; only function order differs (solx alphabetical,
// solc source order), so the CHECK blocks are split per backend.

// CHECK-SOLX: sol.func @{{.*callExternal.*}}(%{{.*}}: !sol.ext_func_ref<(ui256) -> ui256>) -> ui256
// CHECK-SOLX:   sol.ext_icall %{{.*}} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)
// CHECK-SOLX: sol.func @{{.*callInternal.*}}() -> ui256
// CHECK-SOLX:   sol.func_constant @{{.*}} : !sol.func_ref<(ui256) -> ui256>
// CHECK-SOLX:   sol.icall %{{.*}} : !sol.func_ref<(ui256) -> ui256>, (ui256) -> ui256

// CHECK-SOLC: sol.func @{{.*callInternal.*}}() -> ui256
// CHECK-SOLC:   sol.func_constant @{{.*}} : !sol.func_ref<(ui256) -> ui256>
// CHECK-SOLC:   sol.icall %{{.*}} : !sol.func_ref<(ui256) -> ui256>, (ui256) -> ui256
// CHECK-SOLC: sol.func @{{.*callExternal.*}}(%{{.*}}: !sol.ext_func_ref<(ui256) -> ui256>) -> ui256
// CHECK-SOLC:   sol.ext_icall %{{.*}} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)

contract C {
    function g(uint256 x) internal pure returns (uint256) { return x; }
    function callInternal() public pure returns (uint256) {
        function(uint256) internal pure returns (uint256) fp = g;
        return fp(7);
    }

    function callExternal(function(uint256) external returns (uint256) fp) public returns (uint256) {
        return fp(3);
    }
}
