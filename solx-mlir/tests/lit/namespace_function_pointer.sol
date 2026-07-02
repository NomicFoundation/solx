// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A namespace-qualified internal function used as a value (C.g) is an internal function pointer,
// like a bare g: solc's print-init aborts NYI (UNREACHABLE at SolidityToMLIR.cpp:1698), so solx-only.

// CHECK: sol.func @{{.*g.*}}() -> ui256 attributes {{.*}}id = {{[0-9]+}}
// CHECK: sol.func @{{.*run.*}}
// CHECK: sol.func_constant @{{.*g.*}} : !sol.func_ref<() -> ui256>
// CHECK: sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256

contract C {
    function g() internal returns (uint256) {
        return 42;
    }

    function run() public returns (uint256) {
        function () internal returns (uint256) functionPointer = C.g;
        return functionPointer();
    }
}
