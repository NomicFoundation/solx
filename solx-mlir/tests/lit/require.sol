// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*check.*}}
// CHECK:   %[[COND:.*]] = sol.cmp gt
// CHECK:   sol.require %[[COND]]

// CHECK: sol.func @{{.*check_msg.*}}
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
