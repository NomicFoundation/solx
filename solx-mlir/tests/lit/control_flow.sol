// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*if_else.*}}
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.return
// CHECK:   } else {
// CHECK:     sol.return

// CHECK: sol.func @{{.*while_loop.*}}
// CHECK:   sol.while {
// CHECK:     sol.condition %{{.*}}
// CHECK:   } do {
// CHECK:     sol.yield

// For-loop step uses unchecked add (sol.add not sol.cadd)
// CHECK: sol.func @{{.*for_loop.*}}
// CHECK:   sol.for cond {
// CHECK:     sol.condition %{{.*}}
// CHECK:   } body {
// CHECK:     sol.yield
// CHECK:   } step {
// CHECK:     sol.add %
// CHECK:     sol.yield

// CHECK: sol.func @{{.*do_while.*}}
// CHECK:   sol.do {
// CHECK:     sol.yield
// CHECK:   } while {
// CHECK:     sol.condition %{{.*}}

// CHECK: sol.func @{{.*infinite_for.*}}
// CHECK:   sol.for cond {
// CHECK:     %[[TRUE:.*]] = arith.constant true
// CHECK:     sol.condition %[[TRUE]]
// CHECK:   } body {

// CHECK: sol.func @{{.*with_break.*}}
// CHECK:   sol.while {
// CHECK:     sol.condition
// CHECK:   } do {
// CHECK:     sol.if
// CHECK:       sol.break

// CHECK: sol.func @{{.*with_continue.*}}
// CHECK:   sol.while {
// CHECK:     sol.condition
// CHECK:   } do {
// CHECK:     sol.if
// CHECK:       sol.continue

contract C {
    function if_else(uint256 x) public pure returns (uint256) {
        if (x > 10) {
            return 1;
        } else {
            return 0;
        }
    }

    function while_loop(uint256 n) public pure returns (uint256) {
        uint256 sum = 0;
        uint256 i = 0;
        while (i < n) {
            sum = sum + i;
            i = i + 1;
        }
        return sum;
    }

    function for_loop(uint256 n) public pure returns (uint256) {
        uint256 sum = 0;
        for (uint256 i = 0; i < n; i++) {
            sum = sum + i;
        }
        return sum;
    }

    function do_while(uint256 n) public pure returns (uint256) {
        uint256 i = 0;
        do {
            i = i + 1;
        } while (i < n);
        return i;
    }

    function infinite_for() public pure returns (uint256) {
        uint256 i = 0;
        for (;;) {
            i = i + 1;
            if (i == 10) {
                return i;
            }
        }
    }

    function with_break(uint256 n) public pure returns (uint256) {
        uint256 i = 0;
        while (i < n) {
            if (i == 5) {
                break;
            }
            i = i + 1;
        }
        return i;
    }

    function with_continue(uint256 n) public pure returns (uint256) {
        uint256 sum = 0;
        uint256 i = 0;
        while (i < n) {
            i = i + 1;
            if (i == 3) {
                continue;
            }
            sum = sum + i;
        }
        return sum;
    }
}
