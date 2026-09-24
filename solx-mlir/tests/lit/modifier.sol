// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}guarded{{.*}}(%arg0: ui256) -> ui256
// CHECK:   sol.store %arg0, %[[X:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.modifier_invocation @[[BOUNDED:.*]] {
// CHECK:     %[[ARGUMENT:.*]] = sol.load %[[X]]
// CHECK:     sol.yield %[[ARGUMENT]] : ui256
// CHECK:   sol.modifier_invocation @[[TWICE:.*]] {
// CHECK:     sol.yield{{$}}

// CHECK: sol.modifier @[[BOUNDED]](%arg0: ui256) {
// CHECK:   sol.store %arg0, %[[LIMIT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   %[[VALUE:.*]] = sol.load %[[LIMIT]]
// CHECK:   sol.cmp eq, %[[VALUE]]
// CHECK:   sol.if
// CHECK-NEXT: sol.return
// CHECK:   sol.placeholder

// CHECK: sol.modifier @[[TWICE]]() {
// CHECK:   sol.for
// CHECK:   } body {
// CHECK-NEXT: sol.placeholder

// CHECK: sol.func @{{.*}}other{{.*}}(%arg0: ui256) -> ui256
// CHECK:   sol.modifier_invocation @[[BOUNDED]] {
// CHECK-NOT: sol.modifier @[[BOUNDED]](
// CHECK: } {kind = #Contract}

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

    function guarded(uint256 x) public bounded(x) twice returns (uint256) {}

    function other(uint256 y) public bounded(y) returns (uint256) {}
}
