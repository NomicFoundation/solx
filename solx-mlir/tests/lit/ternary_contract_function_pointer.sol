// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A ternary whose branches reference public functions through the contract type
// (`b ? C.f : C.g`), then calls the result. Through the type (not an instance) a
// public function is an internal pointer regardless of visibility, so each branch
// emits a `sol.func_constant` of `func_ref` type (not `ext_func_ref`) and the call
// is a `sol.icall`. No solc parity RUN line: solc's own MLIR lowering NYIs on a
// contract-type member access whose result is a function type.

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
