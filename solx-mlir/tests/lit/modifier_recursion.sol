// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}reached{{.*}}
// CHECK:   sol.modifier_invocation @[[REACHING:.*]] {

// CHECK: sol.modifier @[[REACHING]]() {
// CHECK:   sol.call @[[HELPER:.*]]() : () -> ()
// CHECK:   sol.placeholder

// CHECK: sol.func @[[HELPER]]()
// CHECK:   sol.modifier_invocation @[[REACHING]] {
// CHECK-NOT: sol.modifier @[[REACHING]](
// CHECK: } {kind = #Contract}

contract C {
    modifier reaching() {
        helper();
        _;
    }

    function reached() public reaching {}

    function helper() internal reaching {}
}
