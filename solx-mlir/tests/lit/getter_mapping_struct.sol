// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.state_var @{{.*infos.*}} slot 0 offset 0 : !sol.mapping<ui256, !sol.struct<(ui256, !sol.address), Storage>>

// CHECK: sol.func @{{.*infos.*}}(%arg0: ui256) -> (ui256, !sol.address) attributes {{.*}}selector = 685653147 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*infos.*}} : !sol.mapping<ui256, !sol.struct<(ui256, !sol.address), Storage>>
// CHECK:   %[[S:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<ui256, !sol.struct<(ui256, !sol.address), Storage>>, ui256, !sol.struct<(ui256, !sol.address), Storage>
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[S]], %[[I0]] : !sol.struct<(ui256, !sol.address), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK:   %[[P1:.*]] = sol.gep %[[S]], %[[I1]] : !sol.struct<(ui256, !sol.address), Storage>, ui64, !sol.ptr<!sol.address, Storage>
// CHECK:   %[[V1:.*]] = sol.load %[[P1]] : !sol.ptr<!sol.address, Storage>, !sol.address
// CHECK:   sol.return %[[V0]], %[[V1]] : ui256, !sol.address

contract C {
    struct Info { uint256 a; address b; }
    mapping(uint256 => Info) public infos;
}
