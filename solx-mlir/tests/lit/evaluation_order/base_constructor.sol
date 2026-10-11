// RUN: slang --emit-mlir=sol %evaluation_order/base_constructor.sol | FileCheck %s

// CHECK: sol.func @"@constructor()_{{[0-9]+}}"
// CHECK:   sol.constant 2 : ui8
// CHECK:   %[[SECOND:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.store %[[SECOND]], %[[SECOND_SLOT:.*]] :
// CHECK:   sol.constant 1 : ui8
// CHECK:   %[[FIRST:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.store %[[FIRST]], %[[FIRST_SLOT:.*]] :
// CHECK:   %[[SECOND_ARGUMENT:.*]] = sol.load %[[SECOND_SLOT]] :
// CHECK:   %[[FIRST_ARGUMENT:.*]] = sol.load %[[FIRST_SLOT]] :
// CHECK:   sol.call @"@constructor()_{{[0-9]+}}"(%[[SECOND_ARGUMENT]], %[[FIRST_ARGUMENT]])
