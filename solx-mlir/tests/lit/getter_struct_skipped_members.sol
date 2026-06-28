// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*s.*}}() -> (ui256, !sol.address) attributes {{.*}}selector = -2034821918 : i32
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*s.*}} :
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[BASE]], %[[I0]] : {{.*}}, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I3:.*]] = sol.constant 3 : ui64
// CHECK:   %[[P3:.*]] = sol.gep %[[BASE]], %[[I3]] : {{.*}}, ui64, !sol.ptr<!sol.address, Storage>
// CHECK:   %[[V3:.*]] = sol.load %[[P3]] : !sol.ptr<!sol.address, Storage>, !sol.address
// CHECK:   sol.return %[[V0]], %[[V3]] : ui256, !sol.address

contract C {
    struct S { uint256 a; mapping(uint256 => uint256) m; uint256[] arr; address b; }
    S public s;
}
