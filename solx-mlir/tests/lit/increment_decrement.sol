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

contract C {
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
}
