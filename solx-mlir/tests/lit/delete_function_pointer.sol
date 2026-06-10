// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `delete` of a value lvalue stores the lvalue type's own zero. For a function
// pointer that is `sol.default_func_constant` (a pointer that reverts when
// called), not a `ui256` 0 coerced through an ill-typed integer cast.

// CHECK: sol.func @{{.*reset.*}}
// CHECK: sol.default_func_constant : !sol.func_ref<{{.*}}>
// CHECK: sol.store %{{[0-9]+}}, %{{[0-9]+}} : !sol.func_ref<{{.*}}>, !sol.ptr<!sol.func_ref<{{.*}}>

contract C {
    function g() internal pure returns (uint256) {
        return 1;
    }

    function reset() public pure returns (uint256) {
        function() internal pure returns (uint256) fp = g;
        delete fp;
        return fp == g ? 1 : 0;
    }
}
