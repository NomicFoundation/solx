// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*eq.*}}
// CHECK:   sol.cmp eq, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*ne.*}}
// CHECK:   sol.cmp ne, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*lt.*}}
// CHECK:   sol.cmp lt, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*le.*}}
// CHECK:   sol.cmp le, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*gt.*}}
// CHECK:   sol.cmp gt, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*ge.*}}
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
