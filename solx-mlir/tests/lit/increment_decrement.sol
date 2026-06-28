// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*prefix_inc.*}}
// CHECK-DAG:   %[[OLD:.*]] = sol.load
// CHECK-DAG:   %[[NEW:.*]] = sol.cadd %[[OLD]]
// CHECK-DAG:   sol.store %[[NEW]]
// CHECK-DAG:   sol.return %[[NEW]]

// CHECK-DAG: sol.func @{{.*postfix_inc.*}}
// CHECK-DAG:   %[[OLD:.*]] = sol.load
// CHECK-DAG:   %[[NEW:.*]] = sol.cadd %[[OLD]]
// CHECK-DAG:   sol.store %[[NEW]]
// CHECK-DAG:   sol.return %[[OLD]]

// CHECK-DAG: sol.func @{{.*prefix_dec.*}}
// CHECK-DAG:   %[[OLD:.*]] = sol.load
// CHECK-DAG:   %[[NEW:.*]] = sol.csub %[[OLD]]
// CHECK-DAG:   sol.store %[[NEW]]
// CHECK-DAG:   sol.return %[[NEW]]

// CHECK-DAG: sol.func @{{.*postfix_dec.*}}
// CHECK-DAG:   %[[OLD:.*]] = sol.load
// CHECK-DAG:   %[[NEW:.*]] = sol.csub %[[OLD]]
// CHECK-DAG:   sol.store %[[NEW]]
// CHECK-DAG:   sol.return %[[OLD]]

// Increment / decrement of a STATE variable in discarded statement position, plus
// bare expression statements whose value is unused: each ++/-- combines the loaded
// storage value and stores it back, and `a + a;` / `a;` still emit the computation.
// CHECK-DAG: sol.func @{{.*state_var_stmts.*}}
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 s;

    function prefix_inc(uint256 x) public pure returns (uint256) {
        return ++x;
    }

    function postfix_inc(uint256 x) public pure returns (uint256) {
        return x++;
    }

    function prefix_dec(uint256 x) public pure returns (uint256) {
        return --x;
    }

    function postfix_dec(uint256 x) public pure returns (uint256) {
        return x--;
    }

    function state_var_stmts(uint256 a) public {
        s++;
        s--;
        ++s;
        --s;
        a + a;
        a;
    }
}
