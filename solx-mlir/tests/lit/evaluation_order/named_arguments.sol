// RUN: solx --emit-mlir=sol %evaluation_order/named_arguments.sol | FileCheck %s

// CHECK: sol.func @{{.*call.*}}
// CHECK:   sol.constant 1 : ui8
// CHECK:   %[[A:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.constant 2 : ui8
// CHECK:   %[[B:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.constant 3 : ui8
// CHECK:   %[[C:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.call @{{.*triple.*}}(%[[A]], %[[B]], %[[C]])

// CHECK: sol.func @{{.*struct_constructor.*}}
// CHECK:   sol.constant 1 : ui8
// CHECK:   %[[FIELD_A:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.store %[[FIELD_A]]
// CHECK:   sol.constant 2 : ui8
// CHECK:   %[[FIELD_B:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.store %[[FIELD_B]]
// CHECK:   sol.constant 3 : ui8
// CHECK:   %[[FIELD_C:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.store %[[FIELD_C]]
