// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*pick.*}}(%{{.*}}: ui256) -> ui256
// CHECK: sol.func @{{.*pick.*}}(%{{.*}}: i1) -> i1

// CHECK: sol.func @{{.*sum.*}}(%{{.*}}: ui256) -> ui256
// CHECK: sol.func @{{.*sum.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256

// CHECK: sol.func @{{.*run_nested.*}}
// CHECK:   %[[INNER:.*]] = sol.call @{{.*sum.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK:   sol.call @{{.*sum.*}}(%[[INNER]], %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*get.*}}(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector = -1794649190
// CHECK: sol.func @{{.*get.*}}(%{{.*}}: i1) -> i1 attributes {{.*}}selector = -1044471942

// CHECK: sol.func @{{.*apply_pointer.*}}(%{{.*}}: !sol.func_ref<(ui256) -> ui256>) -> ui256
// CHECK:   sol.icall %{{[0-9]+}}(%{{.*}}) : !sol.func_ref<(ui256) -> ui256>, (ui256) -> ui256
// CHECK: sol.func @{{.*apply_pointer.*}}(%{{.*}}: !sol.func_ref<(i1) -> i1>) -> i1
// CHECK:   sol.icall %{{[0-9]+}}(%{{.*}}) : !sol.func_ref<(i1) -> i1>, (i1) -> i1

// CHECK: sol.func @{{.*run.*}}
// CHECK:   sol.call @{{.*apply_pointer.*}}(%{{.*}}) : (!sol.func_ref<(i1) -> i1>) -> i1
// CHECK:     sol.call @{{.*pick.*}}(%{{.*}}) : (i1) -> i1

contract C {
    function pick(uint256 x) internal pure returns (uint256) {
        return x;
    }

    function pick(bool b) internal pure returns (bool) {
        return b;
    }

    function sum(uint256 x) internal pure returns (uint256) {
        return x;
    }

    function sum(uint256 x, uint256 y) internal pure returns (uint256) {
        return x + y;
    }

    function run_nested() public pure returns (uint256) {
        return sum(sum(10), 20);
    }

    function get(uint256 x) public pure returns (uint256) {
        return x;
    }

    function get(bool b) public pure returns (bool) {
        return b;
    }

    function apply_pointer(function(uint256) internal pure returns (uint256) f) internal pure returns (uint256) {
        return f(1);
    }

    function apply_pointer(function(bool) internal pure returns (bool) f) internal pure returns (bool) {
        return f(true);
    }

    function first(uint256 x) internal pure returns (uint256) {
        return x + 1;
    }

    function second(bool b) internal pure returns (bool) {
        return !b;
    }

    function run() public pure returns (uint256) {
        if (apply_pointer(second) || pick(true)) {
            return apply_pointer(first) + pick(1);
        }
        return 0;
    }
}
