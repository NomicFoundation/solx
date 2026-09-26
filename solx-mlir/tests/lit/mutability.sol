// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*nonpayable_fn.*}}{{.*}} state_mutability = #{{.*}}NonPayable
// CHECK: sol.func @{{.*payable_fn.*}}{{.*}} state_mutability = #{{.*}}Payable
// CHECK: sol.func @{{.*pure_fn.*}}{{.*}} state_mutability = #{{.*}}Pure
// CHECK: sol.func @{{.*view_fn.*}}{{.*}} state_mutability = #{{.*}}View

contract C {
    uint256 x;

    function pure_fn(uint256 a) public pure returns (uint256) {
        return a;
    }

    function view_fn() public view returns (uint256) {
        return x;
    }

    function payable_fn() public payable returns (uint256) {
        return msg.value;
    }

    function nonpayable_fn(uint256 val) public {
        x = val;
    }
}
