// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Constructor
// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Receive, state_mutability = #{{.*}}Payable
// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Fallback, state_mutability = #{{.*}}Payable

contract C {
    uint256 x;

    constructor(uint256 val) {
        x = val;
    }

    receive() external payable {}

    fallback() external payable {}

    function get() public view returns (uint256) {
        return x;
    }
}
