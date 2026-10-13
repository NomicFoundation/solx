// RUN: slang --emit-mlir=sol %evaluation_order/array_literal.sol | FileCheck %s

// CHECK: sol.func @{{.*enumElement.*}}
// CHECK:   %[[FIRST:.*]] = sol.enum_cast %{{.*}} : !sol.enum<1> to !sol.enum<1>
// CHECK:   sol.call @{{.*failChoice.*}}
// CHECK:   %[[SECOND:.*]] = sol.enum_cast %{{.*}} : !sol.enum<1> to !sol.enum<1>
// CHECK:   sol.array_lit %[[FIRST]], %[[SECOND]] :
