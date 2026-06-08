// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Function-pointer `.selector` / `.address` and error / event `.selector`.
// solx-only: solc's MLIR frontend aborts (NYI, SolidityToMLIR.cpp:1518) on every
// direct `.selector` / `.address` member access, so there is no solc RUN line to
// cross-check. A statically-named function / error / event selector folds to a
// compile-time constant bridged to fixedbytes; an external function-pointer
// VALUE pulls its selector / address at runtime via the native Sol ops.

// CHECK-LABEL: sol.func @{{.*}}named_selector
// CHECK: sol.constant {{.*}} : ui32
// CHECK: sol.bytes_cast {{.*}} to !sol.fixedbytes<4>

// CHECK-LABEL: sol.func @{{.*}}pointer_members
// CHECK-DAG: sol.ext_func_selector {{.*}} -> !sol.fixedbytes<4>
// CHECK-DAG: sol.ext_func_addr {{.*}} -> !sol.address

// CHECK-LABEL: sol.func @{{.*}}error_selector
// CHECK: sol.constant {{.*}} : ui32
// CHECK: sol.bytes_cast {{.*}} to !sol.fixedbytes<4>

// CHECK-LABEL: sol.func @{{.*}}event_selector
// CHECK: sol.constant {{.*}} : ui256
// CHECK: sol.bytes_cast {{.*}} to !sol.fixedbytes<32>

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
}
