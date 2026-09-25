// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*check.*}}
// CHECK:   %[[COND:.*]] = sol.cmp gt
// CHECK:   sol.assert %[[COND]]

contract C {
    function check(uint256 x) public pure returns (uint256) {
        assert(x > 0);
        return x;
    }
}
