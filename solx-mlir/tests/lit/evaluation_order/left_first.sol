// RUN: solx --emit-mlir=sol %evaluation_order/left_first.sol | FileCheck %s

// CHECK: sol.func @{{.*logicalAnd\(.*}}
// CHECK:   sol.call @{{.*left.*}}
// CHECK:   sol.if
// CHECK:     sol.call @{{.*right.*}}

// CHECK: sol.func @{{.*logicalOrShortCircuit.*}}
// CHECK:   sol.call @{{.*left.*}}
// CHECK:   sol.if
// CHECK:     sol.call @{{.*right.*}}

// CHECK: sol.func @{{.*conditionalTrue.*}}
// CHECK:   sol.call @{{.*left.*}}
// CHECK:   sol.if
// CHECK:     sol.call @{{.*right.*}}
// CHECK:   } else {
// CHECK:     sol.call @{{.*modulus.*}}

// CHECK: sol.func @{{.*nestedIndex.*}}
// CHECK:   sol.call @{{.*left.*}}
// CHECK:   sol.map
// CHECK:   sol.call @{{.*right.*}}
// CHECK:   sol.map

// CHECK: sol.func @{{.*functionCall.*}}
// CHECK:   %[[CALL_LEFT:.*]] = sol.call @{{.*left.*}}
// CHECK:   %[[CALL_RIGHT:.*]] = sol.call @{{.*right.*}}
// CHECK:   %[[CALL_MODULUS:.*]] = sol.call @{{.*modulus.*}}
// CHECK:   sol.call @{{.*combine.*}}(%[[CALL_LEFT]], %[[CALL_RIGHT]], %[[CALL_MODULUS]])
