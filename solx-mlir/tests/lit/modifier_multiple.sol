// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Multiple modifiers lower to one `sol.modifier_call_blk` each, in source order, at the top of the
// function. A modifier taking an argument carries the function's whole parameter list as its
// isolated block's arguments and references them directly (no spill). Each distinct invoked modifier
// emits exactly one contract-level `sol.modifier` def.

// CHECK: sol.func @{{.*f.*}}(%arg0: ui256) -> ui256 attributes
// CHECK-NEXT: sol.modifier_call_blk {
// CHECK-NEXT: ^bb0(%[[A:.*]]: ui256):
// CHECK-NEXT: sol.call @{{.*onlyPos.*}}(%[[A]]) : (ui256) -> ()
// CHECK-NEXT: }
// CHECK-NEXT: sol.modifier_call_blk {
// CHECK-NEXT: ^bb0(%{{.*}}: ui256):
// CHECK-NEXT: sol.call @{{.*nonReentrant.*}}() : () -> ()
// CHECK-NEXT: }
// CHECK: sol.return
// CHECK: sol.modifier @{{.*onlyPos.*}}(%arg0: ui256) {
// CHECK: sol.require
// CHECK-NEXT: sol.placeholder
// CHECK-NEXT: sol.return
// CHECK: sol.modifier @{{.*nonReentrant.*}}() {
// CHECK-NEXT: sol.placeholder
// CHECK-NEXT: sol.return

contract C {
    uint256 x;
    modifier onlyPos(uint256 v) {
        require(v > 0);
        _;
    }
    modifier nonReentrant() {
        _;
    }
    function f(uint256 a) public onlyPos(a) nonReentrant returns (uint256) {
        return a + x;
    }
}
