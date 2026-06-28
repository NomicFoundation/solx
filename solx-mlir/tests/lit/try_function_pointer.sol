// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

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
