// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// A `{gas: g, value: v}` external call threads `g` as the call's gas operand (capping the forwarded
// gas) and `v` as msg.value, both coerced to ui256 - matching solc. solx previously evaluated `{gas:}`
// but discarded it, forwarding all remaining gas via `sol.gasleft`; it now caps the gas like solc.
// The gas and value operands are now identical between
// the backends; the only remaining divergence is the call op itself: solx emits `sol.ext_icall` (an
// `ext_func_ref` callee), solc a symbol-callee `sol.ext_call`.

// CHECK: sol.constant 5000 : ui16
// CHECK: %[[G:.*]] = sol.cast %{{.*}} : ui16 to ui256
// CHECK: %[[V:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-SOLX: sol.ext_icall %{{.*}}() gas %[[G]] value %[[V]] : !sol.ext_func_ref<() -> ui256>, () -> (i1, ui256)
// CHECK-SOLC: sol.ext_call "{{.*}}"() at %{{.*}} gas %[[G]] value %[[V]] selector %{{.*}} {callee_type = () -> ui256} : !sol.address, () -> (i1, ui256)

interface I {
    function f() external payable returns (uint256);
}

contract C {
    function g(I i) external returns (uint256) {
        return i.f{gas: 5000, value: 1}();
    }
}
