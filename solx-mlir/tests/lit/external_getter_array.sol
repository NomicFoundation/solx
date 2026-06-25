// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// A public dynamic-array getter called externally on `this` (`this.arr(i)`)
// resolves to a `(uint256) -> (element)` getter and forwards the index as the
// sole external argument. Both backends emit `sol.this` + `sol.address_cast`
// and pass the loaded index, but DIVERGE on the call op (solx
// `sol.ext_func_constant` + `sol.ext_icall`, solc symbol `sol.ext_call`).

// CHECK-SOLX: %[[T:.*]] = sol.this : !sol.contract<"C">
// CHECK-SOLX: %[[I:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLX: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<"C"> to !sol.address
// CHECK-SOLX: %[[FR:.*]] = sol.ext_func_constant %[[A]] {selector = {{.*}} : i32} : !sol.address -> !sol.ext_func_ref<(ui256) -> ui256>
// CHECK-SOLX: sol.ext_icall %[[FR]](%[[I]]) gas %{{.*}} value %{{.*}} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)

// CHECK-SOLC: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK-SOLC: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK-SOLC: %[[I:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLC: sol.ext_call "{{.*}}"(%[[I]]) at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256{{.*}}} : !sol.address, (ui256) -> (i1, ui256)

contract C {
    uint256[] public arr;

    function g(uint256 i) external returns (uint256) {
        return this.arr(i);
    }
}
