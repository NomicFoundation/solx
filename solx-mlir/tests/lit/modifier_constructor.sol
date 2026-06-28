// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK: sol.modifier_call_blk {
// CHECK-NEXT: ^bb0(%[[A:.*]]: ui256):
// CHECK-NEXT: sol.call @{{.*setup.*}}(%[[A]]) : (ui256) -> ()
// CHECK-NEXT: }
// CHECK: sol.return
// CHECK: sol.modifier @{{.*setup.*}}(%arg0: ui256) {
// CHECK: sol.require
// CHECK-NEXT: sol.placeholder
// CHECK-NEXT: sol.return

contract C {
    uint256 x = 5;
    modifier setup(uint256 v) {
        require(v > 0);
        _;
    }

    constructor(uint256 a) setup(a) {
        x = a;
    }
}
