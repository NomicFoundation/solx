// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Modifier body with an early return before the placeholder: solc's print-init
// segfaults (exit 139) lowering it, so this is solx-only.

// CHECK: sol.func @{{.*}}f{{.*}}() -> ui256
// CHECK: sol.modifier_call_blk
// CHECK: sol.call @{{.*stop.*}}() : () -> ()
// CHECK: sol.modifier @{{.*stop.*}}() {
// CHECK-NEXT: sol.return
// CHECK-NEXT: }

contract C {
    modifier stop() {
        return;
        _;
    }

    function f() public stop returns (uint256) {
        return 1;
    }
}
