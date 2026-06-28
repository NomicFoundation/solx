// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*g.*}}() -> ui256 attributes {{.*}}id = {{[0-9]+}}
// CHECK: sol.func @{{.*run.*}}
// CHECK: sol.func_constant @{{.*g.*}} : !sol.func_ref<() -> ui256>
// CHECK: sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256

contract C {
    function g() internal returns (uint256) {
        return 42;
    }

    function run() public returns (uint256) {
        function () internal returns (uint256) fp = g;
        return fp();
    }
}
