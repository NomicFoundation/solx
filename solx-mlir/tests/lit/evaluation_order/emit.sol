// RUN: slang --emit-mlir=sol %evaluation_order/emit.sol | FileCheck %s

// CHECK: sol.func @{{.*copy.*}}
// CHECK:   %[[COPY_ARRAY:.*]] = sol.addr_of @array
// CHECK:   %[[COPY_LENGTH:.*]] = sol.call @{{.*grow.*}}
// CHECK:   sol.emit "Copied(uint256[],uint256)" non_indexed = [%[[COPY_ARRAY]], %[[COPY_LENGTH]]]

// CHECK: sol.func @{{.*indexedArguments.*}}
// CHECK:   sol.constant 2 : ui8
// CHECK:   %[[SECOND:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.constant 1 : ui8
// CHECK:   %[[FIRST:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.emit "Indexed(uint256,uint256)" indexed = [%[[FIRST]], %[[SECOND]]]

// CHECK: sol.func @{{.*indexedEnum.*}}
// CHECK:   %[[ENUM:.*]] = sol.enum_cast %{{.*}} : !sol.enum<1> to !sol.enum<1>
// CHECK:   %[[ENUM_VALUE:.*]] = sol.call @{{.*fail.*}}
// CHECK:   sol.emit "Chosen(uint256,uint8)" indexed = [%[[ENUM_VALUE]], %[[ENUM]]]

// CHECK: sol.func @{{.*indexedReference.*}}
// CHECK:   %[[REFERENCE_TOPIC:.*]] = "sol.keccak256"(%{{.*}}) : (!sol.array<? x ui256, Memory>) -> !sol.fixedbytes<32>
// CHECK:   %[[REFERENCE_VALUE:.*]] = sol.call @{{.*mutate.*}}
// CHECK:   sol.emit "Hashed(uint256[],uint256)" indexed = [%[[REFERENCE_TOPIC]]] non_indexed = [%[[REFERENCE_VALUE]]]
