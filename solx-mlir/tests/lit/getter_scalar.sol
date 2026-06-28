// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*value.*}}() -> ui256 attributes {{.*}}selector = 1067774533 : i32
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*value.*}} : !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

// CHECK-LABEL: sol.func @{{.*owner.*}}() -> !sol.address attributes {{.*}}selector = -1918514341 : i32
// CHECK:   %[[OP:.*]] = sol.addr_of @{{.*owner.*}} : !sol.ptr<!sol.address, Storage>
// CHECK:   %[[OV:.*]] = sol.load %[[OP]] : !sol.ptr<!sol.address, Storage>, !sol.address
// CHECK:   sol.return %[[OV]] : !sol.address

contract C {
    uint256 public value;
    address public owner;
}
