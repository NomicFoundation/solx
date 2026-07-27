// RUN: solx --emit-mlir=sol %evaluation_order/assignment.sol | FileCheck %s

// solc print-init evaluates an assignment's place before its value, unlike legacy, so this is solx-only.

// CHECK: sol.func @{{.*scalar.*}}
// CHECK:   sol.call @{{.*rightValue.*}}
// CHECK:   sol.call @{{.*leftIndex.*}}

// CHECK: sol.func @{{.*compound.*}}
// CHECK:   sol.call @{{.*rightValue.*}}
// CHECK:   sol.call @{{.*leftIndex.*}}

// CHECK: sol.func @{{.*destructure.*}}
// CHECK:   %[[RIGHT_FIRST:.*]] = sol.call @{{.*rightFirst.*}}
// CHECK:   %[[RIGHT_SECOND:.*]] = sol.call @{{.*rightSecond.*}}
// CHECK:   sol.call @{{.*leftFirst.*}}
// CHECK:   sol.call @{{.*leftSecond.*}}
// CHECK:   sol.store %[[RIGHT_SECOND]],
// CHECK:   sol.store %[[RIGHT_FIRST]],

// CHECK: sol.func @{{.*tupleStore.*}}
// CHECK:   %[[X:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   %[[ONE:.*]] = sol.constant 1 : ui8
// CHECK:   %[[TWO:.*]] = sol.constant 2 : ui8
// CHECK:   %[[TWO_CAST:.*]] = sol.cast %[[TWO]] : ui8 to ui256
// CHECK:   sol.store %[[TWO_CAST]], %[[X]]
// CHECK:   %[[ONE_CAST:.*]] = sol.cast %[[ONE]] : ui8 to ui256
// CHECK:   sol.store %[[ONE_CAST]], %[[X]]
