// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// Tuple-assign store order: solx stores targets right-to-left (leftmost-wins,
// Solidity semantics); solc print-init stores left-to-right.

// CHECK: sol.func @{{.*assign_from_call.*}}
// CHECK: %[[R:[0-9]+]]:2 = sol.call @{{.*two.*}}()
// CHECK-SOLX: sol.store %[[R]]#1, %{{[0-9]+}}
// CHECK-SOLX: sol.store %[[R]]#0, %{{[0-9]+}}
// CHECK-SOLC: sol.store %[[R]]#0, %{{[0-9]+}}
// CHECK-SOLC: sol.store %[[R]]#1, %{{[0-9]+}}

// CHECK: sol.func @{{.*swap.*}}
// CHECK: %[[V0:[0-9]+]] = sol.load %{{[0-9]+}}
// CHECK: %[[V1:[0-9]+]] = sol.load %{{[0-9]+}}
// CHECK-SOLX: sol.store %[[V1]], %{{[0-9]+}}
// CHECK-SOLX: sol.store %[[V0]], %{{[0-9]+}}
// CHECK-SOLC: sol.store %[[V0]], %{{[0-9]+}}
// CHECK-SOLC: sol.store %[[V1]], %{{[0-9]+}}

contract C {
    function two() internal pure returns (uint256, uint256) {
        return (1, 2);
    }

    function assign_from_call() public pure returns (uint256) {
        uint256 a;
        uint256 b;
        (a, b) = two();
        return a + b;
    }

    function swap(uint256 x, uint256 y) public pure returns (uint256, uint256) {
        (x, y) = (y, x);
        return (x, y);
    }
}
