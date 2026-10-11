// RUN: slang --emit-mlir=sol %evaluation_order/binary.sol | FileCheck %s

// CHECK: sol.func @{{.*addmod.*}}
// CHECK:   %[[ADDMOD_MODULUS:.*]] = sol.call @{{.*modulus.*}}
// CHECK:   %[[ADDMOD_DIVISOR:.*]] = sol.divisor %[[ADDMOD_MODULUS]] : ui256
// CHECK:   %[[ADDMOD_RIGHT:.*]] = sol.call @{{.*right.*}}
// CHECK:   %[[ADDMOD_LEFT:.*]] = sol.call @{{.*left.*}}
// CHECK:   sol.addmod %[[ADDMOD_LEFT]], %[[ADDMOD_RIGHT]], %[[ADDMOD_DIVISOR]] : ui256

// CHECK: sol.func @{{.*arithmetic.*}}
// CHECK:   %[[ARITHMETIC_RIGHT:.*]] = sol.call @{{.*right.*}}
// CHECK:   %[[ARITHMETIC_LEFT:.*]] = sol.call @{{.*left.*}}
// CHECK:   sol.csub %[[ARITHMETIC_LEFT]], %[[ARITHMETIC_RIGHT]] : ui256

// CHECK: sol.func @{{.*bitwise.*}}
// CHECK:   %[[BITWISE_RIGHT:.*]] = sol.call @{{.*right.*}}
// CHECK:   %[[BITWISE_LEFT:.*]] = sol.call @{{.*left.*}}
// CHECK:   sol.xor %[[BITWISE_LEFT]], %[[BITWISE_RIGHT]] : ui256

// CHECK: sol.func @{{.*comparison.*}}
// CHECK:   %[[COMPARISON_RIGHT:.*]] = sol.call @{{.*right.*}}
// CHECK:   %[[COMPARISON_LEFT:.*]] = sol.call @{{.*left.*}}
// CHECK:   sol.cmp gt, %[[COMPARISON_LEFT]], %[[COMPARISON_RIGHT]] : ui256

// CHECK: sol.func @{{.*comparisonEnum.*}}
// CHECK:   %[[COMPARISON_ENUM_RIGHT:.*]] = sol.enum_cast %{{.*}} : !sol.enum<1> to !sol.enum<1>
// CHECK:   sol.call @{{.*failChoice.*}}
// CHECK:   %[[COMPARISON_ENUM_LEFT:.*]] = sol.enum_cast %{{.*}} : !sol.enum<1> to !sol.enum<1>
// CHECK:   sol.cmp eq, %[[COMPARISON_ENUM_LEFT]], %[[COMPARISON_ENUM_RIGHT]] : !sol.enum<1>

// CHECK: sol.func @{{.*exponentiation.*}}
// CHECK:   %[[EXPONENTIATION_RIGHT:.*]] = sol.call @{{.*right.*}}
// CHECK:   %[[EXPONENTIATION_LEFT:.*]] = sol.call @{{.*left.*}}
// CHECK:   sol.cexp %[[EXPONENTIATION_LEFT]], %[[EXPONENTIATION_RIGHT]] : ui256, ui256 -> ui256

// CHECK: sol.func @{{.*mulmod.*}}
// CHECK:   %[[MULMOD_MODULUS:.*]] = sol.call @{{.*modulus.*}}
// CHECK:   %[[MULMOD_DIVISOR:.*]] = sol.divisor %[[MULMOD_MODULUS]] : ui256
// CHECK:   %[[MULMOD_RIGHT:.*]] = sol.call @{{.*right.*}}
// CHECK:   %[[MULMOD_LEFT:.*]] = sol.call @{{.*left.*}}
// CHECK:   sol.mulmod %[[MULMOD_LEFT]], %[[MULMOD_RIGHT]], %[[MULMOD_DIVISOR]] : ui256

// CHECK: sol.func @{{.*shift.*}}
// CHECK:   %[[SHIFT_RIGHT:.*]] = sol.call @{{.*right.*}}
// CHECK:   %[[SHIFT_LEFT:.*]] = sol.call @{{.*left.*}}
// CHECK:   sol.shl %[[SHIFT_LEFT]], %[[SHIFT_RIGHT]] : ui256, ui256
