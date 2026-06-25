// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Calling a public state-variable getter externally on `this` (`this.x()`)
// resolves the auto-generated `() -> (T)` getter and dispatches it as an
// external call against the contract's own address. Both backends emit
// `sol.this` + `sol.address_cast` and carry the getter selector, but they
// DIVERGE on the call op: solx builds a typed `sol.ext_func_constant` +
// `sol.ext_icall`, while solc emits a symbol-callee `sol.ext_call`. The
// receiver address and the getter selector are identical.

// CHECK-SOLX: %[[T:.*]] = sol.this : !sol.contract<"C">
// CHECK-SOLX: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<"C"> to !sol.address
// CHECK-SOLX: %[[FR:.*]] = sol.ext_func_constant %[[A]] {selector = {{.*}} : i32} : !sol.address -> !sol.ext_func_ref<() -> ui256>
// CHECK-SOLX: sol.ext_icall %[[FR]]() gas %{{.*}} value %{{.*}} : !sol.ext_func_ref<() -> ui256>, () -> (i1, ui256)

// CHECK-SOLC: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK-SOLC: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK-SOLC: sol.ext_call "{{.*}}"() at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = () -> ui256{{.*}}} : !sol.address, () -> (i1, ui256)

contract C {
    uint256 public x;

    function g() external returns (uint256) {
        return this.x();
    }
}
