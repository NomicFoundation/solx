// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.state_var @{{.*balances.*}} slot 0 offset 0 : !sol.mapping<!sol.address, ui256>

// CHECK: sol.func @{{.*balances.*}}(%arg0: !sol.address) -> ui256 attributes {{.*}}selector = 669136355 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*balances.*}} : !sol.mapping<!sol.address, ui256>
// CHECK:   %[[SLOT:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<!sol.address, ui256>, !sol.address, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

contract C {
    mapping(address => uint256) public balances;
}
