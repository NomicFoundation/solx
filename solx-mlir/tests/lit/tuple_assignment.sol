// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG because Solidity writes right-to-left so the leftmost write to an

// CHECK: sol.func @{{.*assign_from_call.*}}
// CHECK: %[[R:[0-9]+]]:2 = sol.call @{{.*two.*}}
// CHECK-DAG: sol.store %[[R]]#0, %{{[0-9]+}}
// CHECK-DAG: sol.store %[[R]]#1, %{{[0-9]+}}

// CHECK: sol.func @{{.*swap.*}}
// CHECK: sol.return %{{[0-9]+}}, %{{[0-9]+}} : ui256, ui256

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
