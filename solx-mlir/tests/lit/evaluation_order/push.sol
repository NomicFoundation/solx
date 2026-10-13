// RUN: slang --emit-mlir=sol %evaluation_order/push.sol | FileCheck %s

// CHECK: sol.func @{{.*length.*}}
// CHECK:   %[[LENGTH:.*]] = sol.length
// CHECK:   %[[SLOT:.*]] = sol.push
// CHECK:   sol.store %[[LENGTH]], %[[SLOT]]
