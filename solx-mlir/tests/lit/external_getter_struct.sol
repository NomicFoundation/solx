// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// A public struct getter called externally on `this` (`this.s()`) flattens the
// struct's value members into the getter's external return list (`() ->
// (ui256, ui256)`), so the external call yields a status plus one result per
// member. Both backends emit `sol.this` + `sol.address_cast`; they DIVERGE on
// the call op (solx `sol.ext_func_constant` + `sol.ext_icall`, solc symbol
// `sol.ext_call`), but agree on the multi-result shape.

// CHECK-SOLX: %[[T:.*]] = sol.this : !sol.contract<"C">
// CHECK-SOLX: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<"C"> to !sol.address
// CHECK-SOLX: %[[FR:.*]] = sol.ext_func_constant %[[A]] {selector = {{.*}} : i32} : !sol.address -> !sol.ext_func_ref<() -> (ui256, ui256)>
// CHECK-SOLX: %[[R:.*]]:3 = sol.ext_icall %[[FR]]() gas %{{.*}} value %{{.*}} : !sol.ext_func_ref<() -> (ui256, ui256)>, () -> (i1, ui256, ui256)
// CHECK-SOLX: sol.return %[[R]]#1, %[[R]]#2 : ui256, ui256

// CHECK-SOLC: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK-SOLC: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK-SOLC: sol.ext_call "{{.*}}"() at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = () -> (ui256, ui256){{.*}}} : !sol.address, () -> (i1, ui256, ui256)

contract C {
    struct S { uint256 a; uint256 b; }
    S public s;

    function g() external returns (uint256, uint256) {
        return this.s();
    }
}
