// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}()
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK:   %[[I:.*]] = sol.constant 0 : ui8
// CHECK:   %[[P:.*]] = sol.gep %[[BASE]], %[[I]] : !sol.string<Storage>, ui8, !sol.ptr<!sol.byte, Storage>
// CHECK:   %[[C:.*]] = sol.constant 120 : ui8
// CHECK:   %[[B:.*]] = sol.bytes_cast %[[C]] : ui8 to !sol.byte
// CHECK:   sol.store %[[B]], %[[P]] : !sol.byte, !sol.ptr<!sol.byte, Storage>

contract C {
    bytes data;

    function f() public {
        data[0] = "x";
    }
}
