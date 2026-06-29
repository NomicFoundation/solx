// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}} -> ui256 attributes
// CHECK-NEXT: sol.modifier_call_blk {
// CHECK-NEXT: sol.call @{{.*nonReentrant.*}}() : () -> ()
// CHECK-NEXT: }
// CHECK: sol.return
// CHECK: sol.modifier @{{.*nonReentrant.*}}() {
// CHECK-NEXT: sol.placeholder
// CHECK-NEXT: sol.return
// CHECK-NEXT: }

contract C {
    bool locked;

    modifier nonReentrant() {
        _;
    }

    function f() public nonReentrant returns (uint256) {
        return 1;
    }
}
