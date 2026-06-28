// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// Named arguments on an external member call `inst.f({b: 99, a: 11})` reorder into the
// callee's declaration order, so both backends cast the arguments as `(11, 99)`. The
// external call itself diverges: solx lowers through `sol.ext_func_constant` + `sol.ext_icall`
// (an `ext_func_ref` callee), solc through a symbol-callee `sol.ext_call`.

// CHECK: %[[A:.*]] = sol.cast %c11_ui8 : ui8 to ui256
// CHECK: %[[B:.*]] = sol.cast %c99_ui8 : ui8 to ui256
// CHECK-SOLX: sol.ext_func_constant %{{.*}} {selector = {{.*}}} : !sol.address -> !sol.ext_func_ref<(ui256, ui256) -> ui256>
// CHECK-SOLX: sol.ext_icall %{{.*}}(%[[A]], %[[B]]) gas %{{.*}} value %{{.*}} {static_call} : !sol.ext_func_ref<(ui256, ui256) -> ui256>, (ui256, ui256) -> (i1, ui256)
// CHECK-SOLC: sol.ext_call "{{.*f.*}}"(%[[A]], %[[B]]) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256, ui256) -> ui256, static_call} : !sol.address, (ui256, ui256) -> (i1, ui256)

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
