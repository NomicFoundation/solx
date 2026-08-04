// RUN: solx --emit-mlir=sol %evaluation_order/named_arguments.sol | FileCheck %s

// solc print-init spells an internal callee `@t_28` rather than `@"t(uint256)_28"`, so the symbol
// CHECKs are solx-only. The behavior is pinned against legacy by
// tests/solidity/simple/evaluation_order/named_arguments.sol.

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
