// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[A:.*]] = sol.cast %c11_ui8 : ui8 to ui256
// CHECK: %[[B:.*]] = sol.cast %c99_ui8 : ui8 to ui256
// CHECK: sol.ext_call "{{.*f.*}}"(%[[A]], %[[B]]) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256, ui256) -> ui256, static_call} : !sol.address, (ui256, ui256) -> (i1, ui256)

contract C {
    function ext(A instance) external view returns (uint256) {
        return instance.f({b: 99, a: 11});
    }
}

contract A {
    function f(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}
