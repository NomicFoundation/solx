// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*g.*}}(%arg0: ui256, %arg1: ui256) -> ui256 attributes
// CHECK-NEXT: sol.modifier_call_blk {
// CHECK-NEXT: ^bb0(%[[A:.*]]: ui256, %[[B:.*]]: ui256):
// CHECK-NEXT: sol.constant 7
// CHECK-NEXT: sol.cast
// CHECK-NEXT: sol.cadd %[[B]], %{{.*}} : ui256
// CHECK-NEXT: sol.call @{{.*onlyPos.*}}(%{{.*}}) : (ui256) -> ()
// CHECK-NEXT: }
// CHECK: sol.return
// CHECK: sol.modifier @{{.*onlyPos.*}}(%arg0: ui256) {

contract C {
    modifier onlyPos(uint256 v) {
        require(v > 0);
        _;
    }

    function g(uint256 a, uint256 b) public onlyPos(b + 7) returns (uint256) {
        return a;
    }
}
