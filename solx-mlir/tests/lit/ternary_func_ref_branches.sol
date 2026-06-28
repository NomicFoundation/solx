// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}(%arg0: i1) -> ui256
// CHECK:   %[[SLOT:.*]] = sol.alloca : !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:   sol.if %{{.*}} {
// CHECK:     %[[A:.*]] = sol.func_constant @{{.*a.*}} : !sol.func_ref<() -> ui256>
// CHECK:     sol.store %[[A]], %[[SLOT]] : !sol.func_ref<() -> ui256>, !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:     %[[B:.*]] = sol.func_constant @{{.*b.*}} : !sol.func_ref<() -> ui256>
// CHECK:     sol.store %[[B]], %[[SLOT]] : !sol.func_ref<() -> ui256>, !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:     sol.yield
// CHECK:   }
// CHECK:   %{{.*}} = sol.load %[[SLOT]] : !sol.ptr<!sol.func_ref<() -> ui256>, Stack>, !sol.func_ref<() -> ui256>

contract C {
    function a() internal pure returns (uint) { return 1; }
    function b() internal pure returns (uint) { return 2; }
    function f(bool cond) public pure returns (uint) {
        function() internal pure returns (uint) fp = cond ? a : b;
        return fp();
    }
}
