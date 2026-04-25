// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"receive()"() attributes {kind = #{{.*}}Receive, state_mutability = #{{.*}}Payable}
// CHECK: sol.func @"fallback()"() attributes {kind = #{{.*}}Fallback, state_mutability = #{{.*}}Payable}
// CHECK: sol.func @"constructor()"() attributes {kind = #{{.*}}Constructor

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
