// RUN: solx --emit-mlir=sol %evaluation_order/external_call.sol | FileCheck %s

// solc print-init evaluates a call's options and a pointer call's arguments before the callee,
// unlike legacy, so this is solx-only.

// CHECK: sol.func @{{.*pointerCallee.*}}
// CHECK:   %[[POINTER:.*]] = sol.call @"callee()_{{[0-9]+}}"
// CHECK:   %[[POINTER_ARGUMENT:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.ext_icall %[[POINTER]](%[[POINTER_ARGUMENT]])

// CHECK: sol.func @{{.*receiverOptions.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.call @"receiver()_{{[0-9]+}}"
// CHECK:   %[[RECEIVER_ADDRESS:.*]] = sol.address_cast %[[RECEIVER]]
// CHECK:   %[[VALUE:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   %[[ARGUMENT:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.ext_call "{{.*target.*}}"(%[[ARGUMENT]]) at %[[RECEIVER_ADDRESS]] gas %{{.*}} value %[[VALUE]]

// CHECK: sol.func @{{.*bareReceiverOptions.*}}
// CHECK:   %[[BARE_RECEIVER:.*]] = sol.call @"bareReceiver()_{{[0-9]+}}"
// CHECK:   %[[BARE_VALUE:.*]] = sol.call @"t(uint256)_{{[0-9]+}}"
// CHECK:   sol.bare_call %[[BARE_RECEIVER]] gas %{{.*}} value %[[BARE_VALUE]]

// CHECK: sol.func @{{.*selectorReceiver.*}}
// CHECK:   %{{.*}} = sol.call @"receiver()_{{[0-9]+}}"
// CHECK:   sol.constant 2551302081 : ui32
