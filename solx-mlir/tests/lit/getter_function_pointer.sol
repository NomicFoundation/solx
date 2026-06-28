// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.ext_func_constant %{{.*}} {selector = {{-?[0-9]+}} : i32} : !sol.address -> !sol.ext_func_ref<() -> ui256>

contract C {
    uint256 public value;

    function getterPointer() external view returns (uint256) {
        function() external view returns (uint256) fp = this.value;
        return fp();
    }
}
