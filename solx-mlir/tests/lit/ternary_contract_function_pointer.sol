// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Ternary yielding an external function pointer: solc's print-init aborts NYI
// (UNREACHABLE at SolidityToMLIR.cpp:1698), so this is solx-only.

// CHECK: sol.func @{{.*test.*}}(%arg0: i1) -> ui256
// CHECK:   %[[SLOT:.*]] = sol.alloca : !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:   sol.if %{{.*}} {
// CHECK:     %[[F:.*]] = sol.func_constant @{{.*f.*}} : !sol.func_ref<() -> ui256>
// CHECK:     sol.store %[[F]], %[[SLOT]] : !sol.func_ref<() -> ui256>, !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:     %[[G:.*]] = sol.func_constant @{{.*g.*}} : !sol.func_ref<() -> ui256>
// CHECK:     sol.store %[[G]], %[[SLOT]] : !sol.func_ref<() -> ui256>, !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:     sol.yield
// CHECK:   }
// CHECK:   %[[FP:.*]] = sol.load %[[SLOT]] : !sol.ptr<!sol.func_ref<() -> ui256>, Stack>, !sol.func_ref<() -> ui256>
// CHECK:   %{{.*}} = sol.icall %[[FP]]() : !sol.func_ref<() -> ui256>, () -> ui256

contract C {
    function f() public pure returns (uint256) { return 1; }
    function g() public pure returns (uint256) { return 2; }
    function test(bool b) public pure returns (uint256) {
        return (b ? C.f : C.g)();
    }
}
