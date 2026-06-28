// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// A low-level `catch (bytes memory data)` binds the whole returndata, so the `sol.try`
// fallback region carries a `string<Memory>` block argument alongside the typed Error /
// Panic regions. Two ops diverge from solc. The try's external call: solx emits
// `sol.ext_func_constant` + `sol.ext_icall`, solc a selector `sol.constant` + `sol.ext_call`.
// The implicit return: solc materializes `sol.alloca`/`sol.store`/`sol.load` plus a trailing
// `sol.yield` per region, where solx returns the value straight from SSA. CHECK-SOLX /
// CHECK-SOLC assert each side.

// CHECK: sol.func @{{.*g.*}}({{.*}}) -> ui256
// CHECK: sol.address_cast %{{.*}} : !sol.contract<{{.*}}> to !sol.address
// CHECK-SOLX: sol.ext_func_constant %{{.*}} {selector = {{.*}}} : !sol.address -> !sol.ext_func_ref<() -> ui256>
// CHECK: sol.gasleft : ui256
// CHECK-SOLX: sol.ext_icall %{{.*}}() gas %{{.*}} value %{{.*}} {try_call} : !sol.ext_func_ref<() -> ui256>, () -> (i1, ui256)
// CHECK-SOLC: sol.ext_call "{{.*}}"() at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = () -> ui256, try_call} : !sol.address, () -> (i1, ui256)
// CHECK: sol.try
// CHECK: panic {
// CHECK: ^bb0({{.*}}: ui256):
// CHECK: error {
// CHECK: ^bb0({{.*}}: !sol.string<Memory>):
// CHECK: fallback {
// CHECK: ^bb0({{.*}}: !sol.string<Memory>):
// CHECK-SOLC: sol.yield
// CHECK-SOLC: %[[L:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLC: sol.return %[[L]] : ui256
// CHECK-SOLX: %[[RV:.*]] = sol.constant 0 : ui256
// CHECK-SOLX: sol.return %[[RV]] : ui256

interface I {
    function f() external returns (uint256);
}

contract C {
    function g(I i) external returns (uint256) {
        try i.f() returns (uint256 v) {
            return v;
        } catch Error(string memory reason) {
            return bytes(reason).length;
        } catch Panic(uint256 code) {
            return code;
        } catch (bytes memory data) {
            return data.length;
        }
    }
}
