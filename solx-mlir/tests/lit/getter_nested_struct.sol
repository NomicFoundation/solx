// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}() -> (ui256, !sol.struct<(ui256), Memory>)
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*}} : !sol.struct<(ui256, !sol.struct<(ui256), Storage>), Storage>
// CHECK:   %[[SCALAR:.*]] = sol.gep %[[BASE]], {{.*}} : !sol.struct<(ui256, !sol.struct<(ui256), Storage>), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   sol.load %[[SCALAR]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[NESTED:.*]] = sol.gep %[[BASE]], {{.*}} : !sol.struct<(ui256, !sol.struct<(ui256), Storage>), Storage>, ui64, !sol.struct<(ui256), Storage>
// CHECK:   %[[MEM:.*]] = sol.data_loc_cast %[[NESTED]] : !sol.struct<(ui256), Storage>, !sol.struct<(ui256), Memory>
// CHECK:   sol.return {{.*}}, %[[MEM]] : ui256, !sol.struct<(ui256), Memory>

contract C {
    struct I { uint256 v; }
    struct S { uint256 a; I i; }
    S public s;
}
