// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"eq(uint256,uint256)"
// CHECK:   sol.cmp eq, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"ne(uint256,uint256)"
// CHECK:   sol.cmp ne, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"lt(uint256,uint256)"
// CHECK:   sol.cmp lt, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"le(uint256,uint256)"
// CHECK:   sol.cmp le, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"gt(uint256,uint256)"
// CHECK:   sol.cmp gt, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"ge(uint256,uint256)"
// CHECK:   sol.cmp ge, %{{.*}}, %{{.*}} : ui256

contract C {
    function eq(uint256 a, uint256 b) public pure returns (bool) {
        return a == b;
    }

    function ne(uint256 a, uint256 b) public pure returns (bool) {
        return a != b;
    }

    function lt(uint256 a, uint256 b) public pure returns (bool) {
        return a < b;
    }

    function le(uint256 a, uint256 b) public pure returns (bool) {
        return a <= b;
    }

    function gt(uint256 a, uint256 b) public pure returns (bool) {
        return a > b;
    }

    function ge(uint256 a, uint256 b) public pure returns (bool) {
        return a >= b;
    }
}
