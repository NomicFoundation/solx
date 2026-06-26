// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Named arguments on an external instance member call `inst.f({...})` are
// reordered into the callee's declaration order (`{b: 99, a: 11}` -> `(11, 99)`).
// solc reorders identically; pinned solx-only because solx lowers the external
// call through `sol.ext_func_constant` + `sol.ext_icall` rather than solc's
// symbol-callee `sol.ext_call` (a pre-existing benign divergence).
// CHECK: %[[A:.*]] = sol.cast %c11_ui8
// CHECK: %[[B:.*]] = sol.cast %c99_ui8
// CHECK: sol.ext_icall %{{.*}}(%[[A]], %[[B]])

contract A {
    function f(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}

contract C {
    function ext(A inst) external view returns (uint256) {
        return inst.f({b: 99, a: 11});
    }
}
