// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// A `{gas: g}` call option is evaluated for its side effects but NOT threaded
// into the call in solx (the call forwards all remaining gas via `sol.gasleft`),
// whereas solc caps the call gas with the option value. So both backends emit
// the `gas` constant and fold the `{value: …}` operand identically, but DIVERGE
// on the call op's gas operand: solx passes `gas %gasleft` while solc passes the
// capped `{gas: …}` value. (They also diverge on the call op itself: solx
// `sol.ext_icall`, solc symbol `sol.ext_call`.)

// CHECK-SOLX: sol.constant 5000 : ui16
// CHECK-SOLX: %[[V:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-SOLX: %[[G:.*]] = sol.gasleft : ui256
// CHECK-SOLX: sol.ext_icall %{{.*}}() gas %[[G]] value %[[V]] : !sol.ext_func_ref<() -> ui256>, () -> (i1, ui256)

// CHECK-SOLC: %[[GC:.*]] = sol.constant 5000 : ui16
// CHECK-SOLC: %[[G:.*]] = sol.cast %[[GC]] : ui16 to ui256
// CHECK-SOLC: %[[V:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-SOLC: sol.ext_call "{{.*}}"() at %{{.*}} gas %[[G]] value %[[V]] selector %{{.*}} {callee_type = () -> ui256} : !sol.address, () -> (i1, ui256)

interface I {
    function f() external payable returns (uint256);
}

contract C {
    function g(I i) external returns (uint256) {
        return i.f{gas: 5000, value: 1}();
    }
}
