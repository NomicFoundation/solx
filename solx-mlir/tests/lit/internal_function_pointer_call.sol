// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// An internal function used as a value lowers to `sol.func_constant @f`, whose
// SolToYul lowering is the i256 constant `f.getId()`; the call through the
// pointer is `sol.icall`, lowered to a `yul.switch` over every same-signature
// function's `id`. Both ends read the function's `id` attribute, so each
// referenceable `sol.func` must carry a unique `id` (its slang node id).

// CHECK: sol.func @{{.*g.*}}() -> ui256 attributes {{.*}}id = {{[0-9]+}}
// CHECK: sol.func @{{.*run.*}}
// CHECK: sol.func_constant @{{.*g.*}} : !sol.func_ref<() -> ui256>
// CHECK: sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256

contract C {
    function g() internal returns (uint256) {
        return 42;
    }

    function run() public returns (uint256) {
        function () internal returns (uint256) fp = g;
        return fp();
    }
}
