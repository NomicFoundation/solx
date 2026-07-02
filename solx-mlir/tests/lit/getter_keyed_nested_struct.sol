// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}(%arg0: ui256) -> !sol.struct<(ui256), Memory>
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*}} : !sol.array<? x !sol.struct<(!sol.struct<(ui256), Storage>), Storage>, Storage>
// CHECK:   %[[ELT:.*]] = sol.gep %[[A]], %arg0 {{.*}}: !sol.array<? x !sol.struct<(!sol.struct<(ui256), Storage>), Storage>, Storage>, ui256, !sol.struct<(!sol.struct<(ui256), Storage>), Storage>
// CHECK:   %[[MEMBER:.*]] = sol.gep %[[ELT]], {{.*}} : !sol.struct<(!sol.struct<(ui256), Storage>), Storage>, ui64, !sol.struct<(ui256), Storage>
// CHECK:   %[[MEM:.*]] = sol.data_loc_cast %[[MEMBER]] : !sol.struct<(ui256), Storage>, !sol.struct<(ui256), Memory>
// CHECK:   sol.return %[[MEM]] : !sol.struct<(ui256), Memory>

contract C {
    struct I { uint256 v; }

    struct S { I i; }

    S[] public array;
}
