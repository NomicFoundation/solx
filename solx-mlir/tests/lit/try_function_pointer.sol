// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A `try` over a call through an external function pointer lowers to a `try_call`
// ext_icall on the function-pointer ref and a `sol.try`; the function-pointer
// shape no longer bypasses try/catch (it used to fall through and drop the catch).

// CHECK: ext_icall {{.*}} {try_call} : !sol.ext_func_ref<() -> ui256>
// CHECK: sol.try
// CHECK: fallback {

contract C {
    function() external returns (uint256) fp;

    function f() external returns (uint256) {
        try fp() returns (uint256 x) {
            return x;
        } catch {
            return 0;
        }
    }
}
