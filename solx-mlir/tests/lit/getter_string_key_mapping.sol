// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*scores.*}}(%arg0: !sol.string<Memory>) -> ui256 attributes {{.*}}selector = -846305981 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*scores.*}} : !sol.mapping<!sol.string<{{.*}}>, ui256>
// CHECK:   %[[SLOT:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<!sol.string<{{.*}}>, ui256>, !sol.string<Memory>, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

contract C {
    mapping(string => uint256) public scores;
}
