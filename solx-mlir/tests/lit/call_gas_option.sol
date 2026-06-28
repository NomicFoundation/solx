// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.constant 5000 : ui16
// CHECK: %[[G:.*]] = sol.cast %{{.*}} : ui16 to ui256
// CHECK: %[[V:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK: sol.ext_call "{{.*}}"() at %{{.*}} gas %[[G]] value %[[V]] selector %{{.*}} {callee_type = () -> ui256} : !sol.address, () -> (i1, ui256)

interface I {
    function f() external payable returns (uint256);
}

contract C {
    function g(I i) external returns (uint256) {
        return i.f{gas: 5000, value: 1}();
    }
}
