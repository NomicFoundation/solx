// RUN: solx --emit-mlir=sol %for_loop/step_overflow.sol | FileCheck %s

// CHECK: sol.func @{{.*counter.*}}
// CHECK:   } step {
// CHECK:     sol.cadd %

// CHECK: sol.func @{{.*inclusive.*}}
// CHECK:   } step {
// CHECK:     sol.cadd %

// CHECK: sol.func @{{.*compound.*}}
// CHECK:   } step {
// CHECK:     sol.cadd %

// CHECK: sol.func @{{.*decrement.*}}
// CHECK:   } step {
// CHECK:     sol.csub %

// CHECK: sol.func @{{.*widened.*}}
// CHECK:   } step {
// CHECK:     sol.cadd %

// CHECK: sol.func @{{.*bodyWrite.*}}
// CHECK:   } step {
// CHECK:     sol.cadd %
