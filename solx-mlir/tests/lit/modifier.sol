// RUN: solx --emit-mlir=sol %s | FileCheck --check-prefixes=CHECK,CHECK-SOLX %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}guarded{{.*}}(%arg0: ui256) -> ui256 attributes
// CHECK:   sol.store %arg0
// CHECK:   sol.store %c0_ui256
// CHECK:   sol.modifier_invocation @[[BOUNDED:.*]] {
// CHECK:     sol.load
// CHECK:     sol.yield %{{.*}} : ui256
// CHECK-NEXT: }
// CHECK:   sol.modifier_invocation @[[TWICE:.*]] {
// CHECK-NEXT: sol.yield{{$}}
// CHECK-NEXT: }
// CHECK:   sol.cadd
// CHECK: sol.modifier @[[BOUNDED]](%arg0: ui256) {
// CHECK:   sol.store %arg0
// CHECK:   sol.if
// CHECK:     sol.return
// CHECK:   sol.placeholder
// CHECK: sol.modifier @[[TWICE]]() {
// CHECK:   sol.for
// CHECK:   } body {
// CHECK-NEXT: sol.placeholder
// The second invocation names the modifier defined above and defines no second copy.
// CHECK-SOLX: sol.func @{{.*}}other{{.*}}(%arg0: ui256) -> ui256
// CHECK-SOLX:   sol.modifier_invocation @[[BOUNDED]] {
// CHECK-NOT: sol.modifier @[[BOUNDED]](

contract C {
    modifier bounded(uint256 limit) {
        if (limit == 0) {
            return;
        }
        _;
    }

    modifier twice() {
        for (uint256 round = 0; round < 2; round++) {
            _;
        }
    }

    function guarded(uint256 x) public bounded(x) twice returns (uint256 r) {
        r = x + x;
    }

    function other(uint256 y) public bounded(y) returns (uint256) {
        return y;
    }
}
