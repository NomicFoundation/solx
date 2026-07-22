// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*assign_from_call.*}}
// CHECK:   %[[R:.*]]:2 = sol.call @{{.*two.*}}()
// CHECK:   sol.store %[[R]]#0, %{{.*}}
// CHECK:   sol.store %[[R]]#1, %{{.*}}

// CHECK: sol.func @{{.*swap.*}}
// CHECK:   %[[V0:.*]] = sol.load
// CHECK:   %[[V1:.*]] = sol.load
// CHECK:   sol.store %[[V0]], %{{.*}}
// CHECK:   sol.store %[[V1]], %{{.*}}

// CHECK: sol.func @{{.*conditional_right.*}}
// CHECK:   sol.if
// CHECK:   sol.store %{{.*}}, %[[A:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.store %{{.*}}, %[[B:.*]] : ui256, !sol.ptr<ui256, Stack>

// CHECK: sol.func @{{.*parenthesized.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>

// CHECK: sol.func @{{.*reference_element.*}}
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.array<3 x ui256, Memory>, !sol.array<? x ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>

contract C {
    uint256[] array;

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

    function conditional_right(bool f) public pure returns (uint256, uint256) {
        uint256 a;
        uint256 b;
        (a, b) = f ? (1, 2) : (3, 4);
        return (a, b);
    }

    function parenthesized(uint256 x) public pure returns (uint256) {
        (x) = 7;
        return x;
    }

    function reference_element() public {
        uint256 x;
        (array, x) = ([uint256(1), 2, 3], 5);
    }
}
