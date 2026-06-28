// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK-LABEL: sol.func @{{.*}}error_selector
// CHECK: sol.constant {{.*}} : ui32
// CHECK: sol.bytes_cast {{.*}} to !sol.fixedbytes<4>

// CHECK-LABEL: sol.func @{{.*}}event_selector
// CHECK: sol.constant {{.*}} : ui256
// CHECK: sol.bytes_cast {{.*}} to !sol.fixedbytes<32>

// CHECK-LABEL: sol.func @{{.*}}named_selector
// CHECK: sol.constant {{.*}} : ui32
// CHECK: sol.bytes_cast {{.*}} to !sol.fixedbytes<4>

// CHECK-LABEL: sol.func @{{.*}}pointer_members
// CHECK-DAG: sol.ext_func_selector {{.*}} -> !sol.fixedbytes<4>
// CHECK-DAG: sol.ext_func_addr {{.*}} -> !sol.address

// CHECK-LABEL: sol.func @{{.*}}pointer_value
// CHECK: sol.ext_func_constant {{.*}} -> !sol.ext_func_ref<() -> ui256>

pragma solidity ^0.8.4;

error MyError(uint256 x);

contract C {
    event MyEvent(uint256 indexed a);

    function bar() external returns (uint256) {
        return 1;
    }

    function named_selector() external view returns (bytes4) {
        return this.bar.selector;
    }

    function pointer_members(function() external fp) external pure returns (bytes4, address) {
        return (fp.selector, fp.address);
    }

    function error_selector() external pure returns (bytes4) {
        return MyError.selector;
    }

    function event_selector() external pure returns (bytes32) {
        return MyEvent.selector;
    }

    function pointer_value() external view returns (function() external returns (uint256)) {
        return this.bar;
    }
}
