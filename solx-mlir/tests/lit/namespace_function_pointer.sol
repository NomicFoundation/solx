// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A namespace-qualified internal function used as a value (C.g) is an internal
// function pointer, like a bare g: solc's print-init crashes (SIGSEGV), so this
// is solx-only.

// CHECK: sol.contract @{{.*C.*}} {
// CHECK: sol.func @{{.*g.*}}() -> ui256 attributes {{.*}}id = {{[0-9]+}}
// CHECK: sol.func @{{.*run.*}}
// CHECK:   sol.func_constant @{{.*g.*}} : !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256
// CHECK: sol.func @{{.*qualified_library.*}}
// CHECK:   sol.func_constant @{{.*pick.*}} : !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256
// CHECK: } {kind = #Contract}

// CHECK: sol.contract @{{.*Lib.*}} {
// CHECK: sol.func @{{.*taker.*}}
// CHECK:   sol.func_constant @{{.*pick.*}} : !sol.func_ref<() -> ui256>
// CHECK:   sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256
// CHECK: } {kind = #Library}

contract C {
    function g() internal returns (uint256) {
        return 42;
    }

    function run() public returns (uint256) {
        function () internal returns (uint256) functionPointer = C.g;
        return functionPointer();
    }

    function qualified_library() public pure returns (uint256) {
        function () internal pure returns (uint256) functionPointer = Lib.pick;
        return functionPointer();
    }
}

library Lib {
    function pick() internal pure returns (uint256) {
        return 5;
    }

    function taker() internal pure returns (uint256) {
        function () internal pure returns (uint256) functionPointer = Lib.pick;
        return functionPointer();
    }
}
