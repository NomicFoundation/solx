// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*functionPointerState.*}} slot 0 offset 0 : !sol.func_ref<() -> ui256>

// CHECK: sol.func @{{.*g.*}}() -> ui256 attributes {{.*}}id = {{[0-9]+}}

// CHECK: sol.func @{{.*run.*}}
// CHECK:   %[[G:.*]] = sol.func_constant @{{.*g.*}} : !sol.func_ref<() -> ui256>
// CHECK:   sol.store %[[G]], %[[SLOT:.*]] : !sol.func_ref<() -> ui256>, !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:   %[[POINTER:.*]] = sol.load %[[SLOT]] : !sol.ptr<!sol.func_ref<() -> ui256>, Stack>, !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %[[POINTER]]() : !sol.func_ref<() -> ui256>, () -> ui256

// CHECK: sol.func @{{.*invoke.*}}(%[[ARGUMENT:.*]]: !sol.func_ref<() -> ui256>) -> ui256
// CHECK:   sol.store %[[ARGUMENT]], %[[SLOT:.*]] : !sol.func_ref<() -> ui256>, !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:   %[[POINTER:.*]] = sol.load %[[SLOT]] : !sol.ptr<!sol.func_ref<() -> ui256>, Stack>, !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %[[POINTER]]() : !sol.func_ref<() -> ui256>, () -> ui256

// CHECK: sol.func @{{.*run_argument.*}}
// CHECK:   sol.func_constant @{{.*g.*}} : !sol.func_ref<() -> ui256>
// CHECK:   sol.call @{{.*invoke.*}}(%{{.*}}) : (!sol.func_ref<() -> ui256>) -> ui256

// CHECK: sol.func @{{.*set.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.func_ref<() -> ui256>, !sol.ptr<!sol.func_ref<() -> ui256>, Storage>

// CHECK: sol.func @{{.*run_state.*}}
// CHECK:   sol.addr_of @{{.*functionPointerState.*}} : !sol.ptr<!sol.func_ref<() -> ui256>, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.func_ref<() -> ui256>, Storage>, !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256

// CHECK: sol.func @{{.*run_element.*}}
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<1 x !sol.func_ref<() -> ui256>, Memory>, {{.*}}!sol.ptr<!sol.func_ref<() -> ui256>, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.func_ref<() -> ui256>, Memory>, !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256

// CHECK: sol.func @{{.*run_call_result.*}}
// CHECK:   %[[PICKED:.*]] = sol.call @{{.*pick.*}} : () -> !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %[[PICKED]]() : !sol.func_ref<() -> ui256>, () -> ui256

// CHECK: sol.func @{{.*run_arguments_results.*}}
// CHECK:   %[[PAIR:.*]] = sol.func_constant @{{.*pair.*}} : !sol.func_ref<(ui256, ui256) -> (ui256, ui256)>
// CHECK:   sol.store %[[PAIR]], %[[SLOT:.*]] : !sol.func_ref<(ui256, ui256) -> (ui256, ui256)>, !sol.ptr<!sol.func_ref<(ui256, ui256) -> (ui256, ui256)>, Stack>
// CHECK:   %[[POINTER:.*]] = sol.load %[[SLOT]] : !sol.ptr<!sol.func_ref<(ui256, ui256) -> (ui256, ui256)>, Stack>, !sol.func_ref<(ui256, ui256) -> (ui256, ui256)>
// CHECK:   sol.icall %[[POINTER]](%{{.*}}, %{{.*}}) : !sol.func_ref<(ui256, ui256) -> (ui256, ui256)>, (ui256, ui256) -> (ui256, ui256)

contract C {
    function () internal returns (uint256) functionPointerState;

    function g() internal returns (uint256) {
        return 42;
    }

    function run() public returns (uint256) {
        function () internal returns (uint256) functionPointer = g;
        return functionPointer();
    }

    function invoke(function () internal returns (uint256) f) internal returns (uint256) {
        return f();
    }

    function run_argument() public returns (uint256) {
        return invoke(g);
    }

    function set() public {
        functionPointerState = g;
    }

    function run_state() public returns (uint256) {
        return functionPointerState();
    }

    function run_element() public returns (uint256) {
        function () internal returns (uint256)[1] memory functionPointers = [g];
        return functionPointers[0]();
    }

    function pick() internal returns (function () internal returns (uint256)) {
        return g;
    }

    function run_call_result() public returns (uint256) {
        return pick()();
    }

    function pair(uint256 a, uint256 b) internal returns (uint256, uint256) {
        return (a, b);
    }

    function run_arguments_results() public returns (uint256, uint256) {
        function (uint256, uint256) internal returns (uint256, uint256) functionPointer = pair;
        return functionPointer(7, 9);
    }
}
