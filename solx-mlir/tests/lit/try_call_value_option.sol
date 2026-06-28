// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[VC:.*]] = sol.constant 1 : ui8
// CHECK: %[[V:.*]] = sol.cast %[[VC]] : ui8 to ui256
// CHECK: %[[ST:.*]], %{{.*}} = sol.ext_call "{{.*}}"() at %{{.*}} gas %{{.*}} value %[[V]] selector %{{.*}} {callee_type = () -> ui256, try_call} : !sol.address, () -> (i1, ui256)
// CHECK: sol.try %[[ST]] {

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
