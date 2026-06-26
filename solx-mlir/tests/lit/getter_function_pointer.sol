// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A public state variable used as a value (`fp = this.value`) is its synthesised
// external getter taken as a function pointer: an `sol.ext_func_ref` carrying the
// getter's selector and ABI signature, matching solc.
// CHECK: sol.ext_func_constant %{{.*}} {selector = {{-?[0-9]+}} : i32} : !sol.address -> !sol.ext_func_ref<() -> ui256>

contract C {
    uint256 public value;

    function getterPointer() external view returns (uint256) {
        function() external view returns (uint256) fp = this.value;
        return fp();
    }
}
