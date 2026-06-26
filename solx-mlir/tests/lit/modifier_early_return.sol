// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc --mlir-action=print-init crashes on this valid modifier shape.

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
