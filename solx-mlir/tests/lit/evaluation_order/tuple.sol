// RUN: slang --emit-mlir=sol %evaluation_order/tuple.sol | FileCheck %s

// CHECK: sol.func @{{.*declaration.*}}
// CHECK:   %[[DECLARATION_FIRST:.*]] = sol.addr_of @array
// CHECK:   %[[DECLARATION_SECOND:.*]] = sol.call @{{.*grow.*}}
// CHECK:   sol.data_loc_cast %[[DECLARATION_SECOND]] :
// CHECK:   sol.data_loc_cast %[[DECLARATION_FIRST]] :

// CHECK: sol.func @{{.*returnStatement.*}}
// CHECK:   %[[RETURN_ARRAY:.*]] = sol.addr_of @array
// CHECK:   sol.call @{{.*grow.*}}
// CHECK:   sol.data_loc_cast %[[RETURN_ARRAY]] :
