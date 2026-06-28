// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// A `try recv.f{value: v}(args)` external call captures the `{value: ...}` option
// (via the call-options layer over the member access) and forwards it as the
// call's `value` operand, while the `try` flag routes failure to the catch
// regions instead of reverting. Both backends fold the `{value: 1}` operand and
// emit a `sol.try` with the four catch regions, but DIVERGE on the call op: solx
// builds a `try_call`-flagged `sol.ext_icall`, solc a `try_call`-flagged symbol
// `sol.ext_call`.

// CHECK-SOLX: %[[VC:.*]] = sol.constant 1 : ui8
// CHECK-SOLX: %[[V:.*]] = sol.cast %[[VC]] : ui8 to ui256
// CHECK-SOLX: %[[R:.*]]:2 = sol.ext_icall %{{.*}}() gas %{{.*}} value %[[V]] {try_call} : !sol.ext_func_ref<() -> ui256>, () -> (i1, ui256)
// CHECK-SOLX: sol.try %[[R]]#0 {

// CHECK-SOLC: %[[VC:.*]] = sol.constant 1 : ui8
// CHECK-SOLC: %[[V:.*]] = sol.cast %[[VC]] : ui8 to ui256
// CHECK-SOLC: %[[ST:.*]], %{{.*}} = sol.ext_call "{{.*}}"() at %{{.*}} gas %{{.*}} value %[[V]] selector %{{.*}} {callee_type = () -> ui256, try_call} : !sol.address, () -> (i1, ui256)
// CHECK-SOLC: sol.try %[[ST]] {

interface I {
    function f() external payable returns (uint256);
}

contract C {
    function g(I i) external returns (uint256) {
        try i.f{value: 1}() returns (uint256 v) {
            return v;
        } catch {
            return 0;
        }
    }
}
