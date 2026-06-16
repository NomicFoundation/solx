// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*if_else.*}}
// CHECK-DAG:   sol.if %{{.*}} {
// CHECK-DAG:     sol.return
// CHECK-DAG:   } else {
// CHECK-DAG:     sol.return

// CHECK-DAG: sol.func @{{.*while_loop.*}}
// CHECK-DAG:   sol.while {
// CHECK-DAG:     sol.condition %{{.*}}
// CHECK-DAG:   } do {
// CHECK-DAG:     sol.yield

// For-loop step uses unchecked add (sol.add not sol.cadd)
// CHECK-DAG: sol.func @{{.*for_loop.*}}
// CHECK-DAG:   sol.for cond {
// CHECK-DAG:     sol.condition %{{.*}}
// CHECK-DAG:   } body {
// CHECK-DAG:     sol.yield
// CHECK-DAG:   } step {
// CHECK-DAG:     sol.add %
// CHECK-DAG:     sol.yield

// CHECK-DAG: sol.func @{{.*do_while.*}}
// CHECK-DAG:   sol.do {
// CHECK-DAG:     sol.yield
// CHECK-DAG:   } while {
// CHECK-DAG:     sol.condition %{{.*}}

// CHECK-DAG: sol.func @{{.*infinite_for.*}}
// CHECK-DAG:     sol.constant true

// CHECK-DAG: sol.func @{{.*with_break.*}}
// CHECK-DAG:   sol.while {
// CHECK-DAG:     sol.condition
// CHECK-DAG:   } do {
// CHECK-DAG:     sol.if
// CHECK-DAG:       sol.break

// CHECK-DAG: sol.func @{{.*with_continue.*}}
// CHECK-DAG:   sol.while {
// CHECK-DAG:     sol.condition
// CHECK-DAG:   } do {
// CHECK-DAG:     sol.if
// CHECK-DAG:       sol.continue

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
