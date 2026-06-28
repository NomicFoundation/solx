// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.state_var @{{.*allowance.*}} slot 0 offset 0 : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>

// CHECK: sol.func @{{.*allowance.*}}(%arg0: !sol.address, %arg1: ui256) -> ui256 attributes {{.*}}selector = -574185103 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*allowance.*}} : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>
// CHECK:   %[[M1:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>, !sol.address, !sol.mapping<ui256, ui256>
// CHECK:   %[[SLOT:.*]] = sol.map %[[M1]], %arg1 : !sol.mapping<ui256, ui256>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

contract C {
    mapping(address => mapping(uint256 => uint256)) public allowance;
}
