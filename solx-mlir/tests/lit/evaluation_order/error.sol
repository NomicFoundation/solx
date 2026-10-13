// RUN: slang --emit-mlir=sol %evaluation_order/error.sol | FileCheck %s

// CHECK: sol.func @{{.*requireError.*}}
// CHECK:   %[[REQUIRE_ARRAY:.*]] = sol.addr_of @array
// CHECK:   %[[REQUIRE_LENGTH:.*]] = sol.call @{{.*grow.*}}
// CHECK:   sol.require %{{.*}}, "Copied(uint256[],uint256)"(%[[REQUIRE_ARRAY]], %[[REQUIRE_LENGTH]] :

// CHECK: sol.func @{{.*revertError.*}}
// CHECK:   %[[REVERT_ARRAY:.*]] = sol.addr_of @array
// CHECK:   %[[REVERT_LENGTH:.*]] = sol.call @{{.*grow.*}}
// CHECK:   sol.revert "Copied(uint256[],uint256)" %[[REVERT_ARRAY]], %[[REVERT_LENGTH]] :
