// RUN: solx --emit-mlir=sol %evaluation_order/nested_assignment.sol | FileCheck %s

// solc print-init resolves an assignment place before its value while solx is value-first to match legacy.

// CHECK: sol.func @{{.*binary.*}}
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"

// CHECK: sol.func @{{.*ternary.*}}
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.if
// CHECK:     sol.constant 3 : ui8
// CHECK:     sol.call @"t(uint256)"
// CHECK:     sol.constant 4 : ui8
// CHECK:     sol.call @"t(uint256)"
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"

// CHECK: sol.func @{{.*call.*}}
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"

// CHECK: sol.func @{{.*indexPlace.*}}
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.call @"t(uint256)"

// CHECK: sol.func @{{.*compound.*}}
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"

// CHECK: sol.func @{{.*tuple.*}}
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 4 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.call @"t(uint256)"
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.call @"t(uint256)"
