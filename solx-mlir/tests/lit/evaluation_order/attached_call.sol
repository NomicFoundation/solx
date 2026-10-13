// RUN: slang --emit-mlir=sol %evaluation_order/attached_call.sol | FileCheck %s

// CHECK: sol.func @{{.*libraryReceiverCalldata.*}}
// CHECK:   %[[COPY:.*]] = sol.data_loc_cast
// CHECK:   %[[COPY_ARGUMENT:.*]] = sol.call @{{.*fail.*}}
// CHECK:   sol.ext_call "{{.*size.*}}"(%[[COPY]], %[[COPY_ARGUMENT]])

// CHECK: sol.func @{{.*libraryReceiverEnum.*}}
// CHECK:   %[[LIBRARY_ENUM:.*]] = sol.enum_cast %{{.*}} : !sol.enum<1> to !sol.enum<1>
// CHECK:   %[[LIBRARY_ENUM_ARGUMENT:.*]] = sol.call @{{.*fail.*}}
// CHECK:   sol.ext_call "{{.*linkedOrdinal.*}}"(%[[LIBRARY_ENUM]], %[[LIBRARY_ENUM_ARGUMENT]])

// CHECK: sol.func @{{.*receiver.*}}
// CHECK:   sol.constant 2 : ui8
// CHECK:   %[[ARGUMENT:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.constant 1 : ui8
// CHECK:   %[[RECEIVER:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.call @{{.*pair.*}}(%[[RECEIVER]], %[[ARGUMENT]])

// CHECK: sol.func @{{.*receiverEnum.*}}
// CHECK:   %[[ENUM:.*]] = sol.enum_cast %{{.*}} : !sol.enum<1> to !sol.enum<1>
// CHECK:   sol.call @{{.*ordinal.*}}(%[[ENUM]])
