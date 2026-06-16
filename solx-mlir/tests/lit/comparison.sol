// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*eq.*}}
// CHECK-DAG:   sol.cmp eq, %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*ne.*}}
// CHECK-DAG:   sol.cmp ne, %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*lt.*}}
// CHECK-DAG:   sol.cmp lt, %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*le.*}}
// CHECK-DAG:   sol.cmp le, %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*gt.*}}
// CHECK-DAG:   sol.cmp gt, %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*ge.*}}
// CHECK-DAG:   sol.cmp ge, %{{.*}}, %{{.*}} : ui256

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
