// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A function with a single no-argument modifier lowers to one `sol.modifier_call_blk` at the top of
// the function (calling the modifier) and one contract-level `sol.modifier` whose `_;` is a
// `sol.placeholder`. The function body is emitted inline, after the call block.

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
