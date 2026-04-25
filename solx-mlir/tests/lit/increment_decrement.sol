// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"prefix_inc(uint256)"
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.cadd %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @"postfix_inc(uint256)"
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.cadd %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @"prefix_dec(uint256)"
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.csub %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @"postfix_dec(uint256)"
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.csub %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[OLD]]

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
