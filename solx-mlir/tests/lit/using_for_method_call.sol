// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// using-for x.add(3): solx loads the receiver and forwards it, sol.call @add(%x, %3) : (ui256, ui256) -> ui256.
// solc print-init drops the receiver, lowering to sol.call @add(%3) : (ui256) -> ui256.

// CHECK: sol.func @{{.*}}(%arg0: ui256) -> ui256
// CHECK: sol.store %arg0, %[[SLOT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK-SOLX: %[[X:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.constant 3 : ui8
// CHECK: %[[THREE:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-SOLX: sol.call @{{.*add.*}}(%[[X]], %[[THREE]]) : (ui256, ui256) -> ui256
// CHECK-SOLC: sol.call @{{.*add.*}}(%[[THREE]]) : (ui256) -> ui256

library L {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }
}

contract C {
    using L for uint256;

    function f(uint256 x) public pure returns (uint256) {
        return x.add(3);
    }
}
