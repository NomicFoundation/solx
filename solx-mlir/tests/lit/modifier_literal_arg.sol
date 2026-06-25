// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A modifier invocation with a literal argument evaluates the literal inside the
// `sol.modifier_call_blk`, casts it to the modifier's parameter type, and passes the result to the
// `sol.call`. The function takes no parameters, so the isolated block has no block arguments.

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
