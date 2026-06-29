// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*reset.*}}
// CHECK: sol.default_func_constant : !sol.func_ref<{{.*}}>
// CHECK: sol.store %{{[0-9]+}}, %{{[0-9]+}} : !sol.func_ref<{{.*}}>, !sol.ptr<!sol.func_ref<{{.*}}>

contract C {
    function g() internal pure returns (uint256) {
        return 1;
    }

    function reset() public pure returns (uint256) {
        function() internal pure returns (uint256) functionPointer = g;
        delete functionPointer;
        return functionPointer == g ? 1 : 0;
    }
}
