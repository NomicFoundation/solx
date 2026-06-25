// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A call-options expression in value position (`i.f{value: v}` decorated but
// not immediately called — here bound to an external function-pointer local)
// contributes only its options' side effects; its value is the wrapped
// operand's. The `{value: v}` is evaluated (the `sol.load` of `v`) but, since
// the decorated pointer is later called separately, the value is NOT threaded —
// the eventual `sol.ext_icall` forwards `value %c0_ui256`. solx-only: solc's
// frontend rejects binding the decorated external function to a local.

// CHECK-LABEL: sol.func @{{.*}}g
// CHECK: %[[V:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.ext_func_constant %{{.*}} {selector = {{.*}} : i32} : !sol.address -> !sol.ext_func_ref<() -> ui256>
// CHECK: %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK: sol.ext_icall %{{.*}}() gas %{{.*}} value %[[ZERO]] : !sol.ext_func_ref<() -> ui256>, () -> (i1, ui256)

interface I {
    function f() external payable returns (uint256);
}

contract C {
    function g(I i, uint256 v) external returns (uint256) {
        function() external payable returns (uint256) fp = i.f{value: v};
        return fp();
    }
}
