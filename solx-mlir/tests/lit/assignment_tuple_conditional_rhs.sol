// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// (a, b) = f ? (1, 2) : (3, 4) destructure store order: solx casts/stores right-to-left (b
// before a); solc print-init casts/stores left-to-right (a before b).

// CHECK: sol.func @{{.*}}condition{{.*}}(%arg0: i1) -> (ui256, ui256)
// CHECK: %[[A:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK: %[[B:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK: sol.if
// CHECK: %[[L0:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui8, Stack>, ui8
// CHECK: %[[L1:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui8, Stack>, ui8
// CHECK-SOLX: %[[S0:.*]] = sol.cast %[[L1]] : ui8 to ui256
// CHECK-SOLX: sol.store %[[S0]], %[[B]] : ui256, !sol.ptr<ui256, Stack>
// CHECK-SOLX: %[[S1:.*]] = sol.cast %[[L0]] : ui8 to ui256
// CHECK-SOLX: sol.store %[[S1]], %[[A]] : ui256, !sol.ptr<ui256, Stack>
// CHECK-SOLC: %[[S0:.*]] = sol.cast %[[L0]] : ui8 to ui256
// CHECK-SOLC: sol.store %[[S0]], %[[A]] : ui256, !sol.ptr<ui256, Stack>
// CHECK-SOLC: %[[S1:.*]] = sol.cast %[[L1]] : ui8 to ui256
// CHECK-SOLC: sol.store %[[S1]], %[[B]] : ui256, !sol.ptr<ui256, Stack>

contract C {
    function condition(bool f) public pure returns (uint256, uint256) {
        uint256 a; uint256 b;
        (a, b) = f ? (1, 2) : (3, 4);
        return (a, b);
    }
}
