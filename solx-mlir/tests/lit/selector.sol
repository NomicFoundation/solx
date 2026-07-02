// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Error and event .selector are compile-time constants: solc's print-init hits NYI and
// aborts (UNREACHABLE at SolidityToMLIR.cpp:813), so this is solx-only.

// CHECK-LABEL: sol.func @{{.*}}error_selector
// CHECK: sol.constant {{.*}} : ui32
// CHECK: sol.bytes_cast {{.*}} : ui32 to !sol.fixedbytes<4>

// CHECK-LABEL: sol.func @{{.*}}event_selector
// CHECK: sol.constant {{.*}} : ui256
// CHECK: sol.bytes_cast {{.*}} : ui256 to !sol.fixedbytes<32>

error MyError(uint256 x);

contract C {
    event MyEvent(uint256 indexed a);

    function error_selector() external pure returns (bytes4) {
        return MyError.selector;
    }

    function event_selector() external pure returns (bytes32) {
        return MyEvent.selector;
    }
}
