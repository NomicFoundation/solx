// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*eq.*}}
// CHECK:   sol.cmp eq, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*eq_mixed.*}}
// CHECK:   sol.cast %{{.*}} : si8 to si16
// CHECK:   sol.cmp eq, %{{.*}}, %{{.*}} : si16

// CHECK: sol.func @{{.*ge.*}}
// CHECK:   sol.cmp ge, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*ge_mixed.*}}
// CHECK:   sol.cast %{{.*}} : si8 to si16
// CHECK:   sol.cmp ge, %{{.*}}, %{{.*}} : si16

// CHECK: sol.func @{{.*gt.*}}
// CHECK:   sol.cmp gt, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*gt_mixed.*}}
// CHECK:   sol.cast %{{.*}} : si8 to si16
// CHECK:   sol.cmp gt, %{{.*}}, %{{.*}} : si16

// CHECK: sol.func @{{.*le.*}}
// CHECK:   sol.cmp le, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*le_mixed.*}}
// CHECK:   sol.cast %{{.*}} : si8 to si16
// CHECK:   sol.cmp le, %{{.*}}, %{{.*}} : si16

// CHECK: sol.func @{{.*lt.*}}
// CHECK:   sol.cmp lt, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*lt_mixed.*}}
// CHECK:   sol.cast %{{.*}} : si8 to si16
// CHECK:   sol.cmp lt, %{{.*}}, %{{.*}} : si16

// CHECK: sol.func @{{.*ne.*}}
// CHECK:   sol.cmp ne, %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*ne_mixed.*}}
// CHECK:   sol.cast %{{.*}} : si8 to si16
// CHECK:   sol.cmp ne, %{{.*}}, %{{.*}} : si16

contract C {
    function eq(uint256 a, uint256 b) public pure returns (bool) {
        return a == b;
    }

    function eq_mixed(int8 a, int16 b) public pure returns (bool) {
        return a == b;
    }

    function ge(uint256 a, uint256 b) public pure returns (bool) {
        return a >= b;
    }

    function ge_mixed(int8 a, int16 b) public pure returns (bool) {
        return a >= b;
    }

    function gt(uint256 a, uint256 b) public pure returns (bool) {
        return a > b;
    }

    function gt_mixed(int8 a, int16 b) public pure returns (bool) {
        return a > b;
    }

    function le(uint256 a, uint256 b) public pure returns (bool) {
        return a <= b;
    }

    function le_mixed(int8 a, int16 b) public pure returns (bool) {
        return a <= b;
    }

    function lt(uint256 a, uint256 b) public pure returns (bool) {
        return a < b;
    }

    function lt_mixed(int8 a, int16 b) public pure returns (bool) {
        return a < b;
    }

    function ne(uint256 a, uint256 b) public pure returns (bool) {
        return a != b;
    }

    function ne_mixed(int8 a, int16 b) public pure returns (bool) {
        return a != b;
    }
}
