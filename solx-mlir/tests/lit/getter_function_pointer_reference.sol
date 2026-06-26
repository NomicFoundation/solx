// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A function pointer to a reference-typed (`string`) public getter carries the
// getter's ForceMemory ABI signature (`() -> !sol.string<Memory>`), matching solc.
// CHECK: sol.ext_func_constant %{{.*}} {selector = {{-?[0-9]+}} : i32} : !sol.address -> !sol.ext_func_ref<() -> !sol.string<Memory>>

contract C {
    string public name;

    function ptr() external view returns (string memory) {
        function() external view returns (string memory) fp = this.name;
        return fp();
    }
}
