// RUN: solx --emit-mlir=sol %s | FileCheck --check-prefixes=CHECK,CHECK-SOLX %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A modifier whose body reaches a function it decorates is defined once: the definition is
// recorded before its body is emitted.
// CHECK: sol.func @{{.*}}reached{{.*}}
// CHECK:   sol.modifier_invocation @[[REACHING:.*]] {
// CHECK: sol.modifier @[[REACHING]]() {
// CHECK:   sol.call @{{.*}}helper{{.*}}
// CHECK:   sol.placeholder
// CHECK-NEXT: sol.return
// CHECK-SOLX: sol.func @{{.*}}helper{{.*}}
// CHECK-SOLX-NEXT: sol.modifier_invocation @[[REACHING]] {
// CHECK-NOT: sol.modifier @[[REACHING]](

contract C {
    modifier reaching() {
        helper();
        _;
    }

    function reached() public reaching {}

    function helper() internal reaching {}
}
