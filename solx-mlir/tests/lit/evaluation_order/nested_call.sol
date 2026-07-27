// RUN: solx --emit-mlir=sol %evaluation_order/nested_call.sol | FileCheck %s

// solc print-init orders binary operands left-first while solx is right-first to match legacy.

// CHECK: sol.func @{{.*binary.*}}
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.call @"t(uint256)"

// CHECK: sol.func @{{.*ternary.*}}
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.if
// CHECK:     sol.constant 2 : ui8
// CHECK:     sol.call @"t(uint256)"
// CHECK:     sol.constant 3 : ui8
// CHECK:     sol.call @"t(uint256)"
// CHECK:   sol.constant 4 : ui8
// CHECK:   sol.call @"t(uint256)"

// CHECK: sol.func @{{.*assignment.*}}
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.call @"t(uint256)"
