// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*nonpayable_function.*}}{{.*}} state_mutability = #NonPayable
// CHECK: sol.func @{{.*payable_function.*}}{{.*}} state_mutability = #Payable
// CHECK: sol.func @{{.*pure_function.*}}{{.*}} state_mutability = #Pure
// CHECK: sol.func @{{.*view_function.*}}{{.*}} state_mutability = #View

contract C {
    uint256 x;

    function nonpayable_function(uint256 value) public {
        x = value;
    }

    function payable_function() public payable returns (uint256) {
        return msg.value;
    }

    function pure_function(uint256 a) public pure returns (uint256) {
        return a;
    }

    function view_function() public view returns (uint256) {
        return x;
    }
}
