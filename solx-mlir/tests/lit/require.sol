// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"check(uint256)"
// CHECK:   %[[COND:.*]] = sol.cmp gt
// CHECK:   sol.require %[[COND]]

// CHECK: sol.func @"check_msg(uint256)"
// CHECK:   %[[COND:.*]] = sol.cmp gt
// CHECK:   sol.require %[[COND]]

contract C {
    function check(uint256 x) public pure returns (uint256) {
        require(x > 0);
        return x;
    }

    function check_msg(uint256 x) public pure returns (uint256) {
        require(x > 0, "must be positive");
        return x;
    }
}
