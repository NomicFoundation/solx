// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Error and event .selector are compile-time constants: solc's print-init
// crashes (SIGSEGV), so this is solx-only.

// CHECK: sol.func @{{.*error_selector.*}}
// CHECK:   sol.constant 816952677 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

// CHECK: sol.func @{{.*event_selector.*}}
// CHECK:   sol.constant 48926247962583432353061649299097379571640188309989729032230735130183305912964 : ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>

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
