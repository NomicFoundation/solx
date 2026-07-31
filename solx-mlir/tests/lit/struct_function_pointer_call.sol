// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*run.*}}
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*}} : !sol.struct<(!sol.func_ref<() -> ui256>), Storage>
// CHECK:   %[[FIELD:.*]] = sol.gep %[[BASE]]{{.*}} : !sol.struct<(!sol.func_ref<() -> ui256>), Storage>, {{.*}}!sol.ptr<!sol.func_ref<() -> ui256>, Storage>
// CHECK:   %[[POINTER:.*]] = sol.load %[[FIELD]] : !sol.ptr<!sol.func_ref<() -> ui256>, Storage>, !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %[[POINTER]]() : !sol.func_ref<() -> ui256>, () -> ui256

contract C {
    struct S {
        function () internal returns (uint256) f;
    }

    S s;

    function g() internal returns (uint256) {
        return 42;
    }

    function set() public {
        s.f = g;
    }

    function run() public returns (uint256) {
        return s.f();
    }
}
