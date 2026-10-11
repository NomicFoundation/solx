// RUN: slang --emit-mlir=sol %evaluation_order/encode_with_signature.sol | FileCheck %s

// CHECK: sol.func @{{.*storageSignature.*}}
// CHECK:   %[[SIGNATURE:.*]] = sol.addr_of @signature
// CHECK:   %[[ARGUMENT:.*]] = sol.call @{{.*rename.*}}
// CHECK:   sol.encode selector(%[[SIGNATURE]]) %[[ARGUMENT]] : !sol.string<Storage> ui256 : !sol.string<Memory>
