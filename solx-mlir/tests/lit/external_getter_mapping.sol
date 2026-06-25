// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// A public mapping getter called externally on `this` (`this.m(k)`) resolves to
// a `(K) -> (V)` getter and lowers the key `k` as the single external argument.
// Both backends emit `sol.this` + `sol.address_cast` and forward the loaded key
// as the call argument, but DIVERGE on the call op: solx uses a typed
// `sol.ext_func_constant` + `sol.ext_icall`, solc a symbol-callee `sol.ext_call`.

// CHECK-SOLX: %[[T:.*]] = sol.this : !sol.contract<"C">
// CHECK-SOLX: %[[K:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLX: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<"C"> to !sol.address
// CHECK-SOLX: %[[FR:.*]] = sol.ext_func_constant %[[A]] {selector = {{.*}} : i32} : !sol.address -> !sol.ext_func_ref<(ui256) -> ui256>
// CHECK-SOLX: sol.ext_icall %[[FR]](%[[K]]) gas %{{.*}} value %{{.*}} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)

// CHECK-SOLC: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK-SOLC: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK-SOLC: %[[K:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLC: sol.ext_call "{{.*}}"(%[[K]]) at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256{{.*}}} : !sol.address, (ui256) -> (i1, ui256)

contract C {
    mapping(uint256 => uint256) public m;

    function g(uint256 k) external returns (uint256) {
        return this.m(k);
    }
}
