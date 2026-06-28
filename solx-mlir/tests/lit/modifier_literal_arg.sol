// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}() -> ui256 attributes
// CHECK-NEXT: sol.modifier_call_blk {
// CHECK-NEXT: sol.constant 7
// CHECK-NEXT: sol.cast
// CHECK-NEXT: sol.call @{{.*atLeast.*}}(%{{.*}}) : (ui256) -> ()
// CHECK-NEXT: }
// CHECK: sol.return
// CHECK: sol.modifier @{{.*atLeast.*}}(%arg0: ui256) {
// CHECK: sol.placeholder
// CHECK-NEXT: sol.return

contract C {
    modifier atLeast(uint256 v) {
        require(v >= 1);
        _;
    }

    function f() public atLeast(7) returns (uint256) {
        return 0;
    }
}
