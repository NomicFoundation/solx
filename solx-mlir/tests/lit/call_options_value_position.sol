// RUN: solx --emit-mlir=sol %s | FileCheck %s

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
