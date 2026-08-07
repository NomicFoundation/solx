// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*origin.*}} slot 0 offset 0 : !sol.struct<(ui256, ui256), Storage>

// CHECK: sol.func @{{.*origin.*}}() -> (ui256, ui256) attributes {{.*}}selector = -1819582670 : i32
// CHECK:   %[[S:.*]] = sol.addr_of @{{.*origin.*}} : !sol.struct<(ui256, ui256), Storage>
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[S]], %[[I0]] : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK:   %[[P1:.*]] = sol.gep %[[S]], %[[I1]] : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V1:.*]] = sol.load %[[P1]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V0]], %[[V1]] : ui256, ui256

// CHECK: sol.func @{{.*s.*}}() -> (ui256, !sol.address) attributes {{.*}}selector = -2034821918 : i32
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*s.*}} :
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[BASE]], %[[I0]] : {{.*}}, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I3:.*]] = sol.constant 3 : ui64
// CHECK:   %[[P3:.*]] = sol.gep %[[BASE]], %[[I3]] : {{.*}}, ui64, !sol.ptr<!sol.address, Storage>
// CHECK:   %[[V3:.*]] = sol.load %[[P3]] : !sol.ptr<!sol.address, Storage>, !sol.address
// CHECK:   sol.return %[[V0]], %[[V3]] : ui256, !sol.address

// CHECK: sol.func @{{.*s.*}}() -> (ui256, !sol.string<Memory>) attributes {{.*}}selector = -2034821918 : i32
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*s.*}} : !sol.struct<(ui256, !sol.string<Storage>), Storage>
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[BASE]], %[[I0]] : !sol.struct<(ui256, !sol.string<Storage>), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK:   %[[P1:.*]] = sol.gep %[[BASE]], %[[I1]] : !sol.struct<(ui256, !sol.string<Storage>), Storage>, ui64, !sol.string<Storage>
// CHECK:   %[[V1:.*]] = sol.data_loc_cast %[[P1]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.return %[[V0]], %[[V1]] : ui256, !sol.string<Memory>

// CHECK: sol.func @{{.*}}() -> (ui256, !sol.struct<(ui256), Memory>)
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*}} : !sol.struct<(ui256, !sol.struct<(ui256), Storage>), Storage>
// CHECK:   %[[SCALAR:.*]] = sol.gep %[[BASE]], {{.*}} : !sol.struct<(ui256, !sol.struct<(ui256), Storage>), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   sol.load %[[SCALAR]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[NESTED:.*]] = sol.gep %[[BASE]], {{.*}} : !sol.struct<(ui256, !sol.struct<(ui256), Storage>), Storage>, ui64, !sol.struct<(ui256), Storage>
// CHECK:   %[[MEM:.*]] = sol.data_loc_cast %[[NESTED]] : !sol.struct<(ui256), Storage>, !sol.struct<(ui256), Memory>
// CHECK:   sol.return {{.*}}, %[[MEM]] : ui256, !sol.struct<(ui256), Memory>

// CHECK: sol.func @{{.*}}(%arg0: ui256) -> !sol.struct<(ui256), Memory>
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*}} : !sol.array<? x !sol.struct<(!sol.struct<(ui256), Storage>), Storage>, Storage>
// CHECK:   %[[ELT:.*]] = sol.gep %[[A]], %arg0 {{.*}}: !sol.array<? x !sol.struct<(!sol.struct<(ui256), Storage>), Storage>, Storage>, ui256, !sol.struct<(!sol.struct<(ui256), Storage>), Storage>
// CHECK:   %[[MEMBER:.*]] = sol.gep %[[ELT]], {{.*}} : !sol.struct<(!sol.struct<(ui256), Storage>), Storage>, ui64, !sol.struct<(ui256), Storage>
// CHECK:   %[[MEM:.*]] = sol.data_loc_cast %[[MEMBER]] : !sol.struct<(ui256), Storage>, !sol.struct<(ui256), Memory>
// CHECK:   sol.return %[[MEM]] : !sol.struct<(ui256), Memory>

// CHECK: sol.func @{{.*items.*}}(%arg0: ui256) -> (ui256, i1) attributes {{.*}}selector = -1078840878 : i32
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*items.*}} : !sol.array<? x !sol.struct<(ui256, i1), Storage>, Storage>
// CHECK:   %[[E:.*]] = sol.gep %[[A]], %arg0 no_panic_bounds : !sol.array<? x !sol.struct<(ui256, i1), Storage>, Storage>, ui256, !sol.struct<(ui256, i1), Storage>
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[E]], %[[I0]] : !sol.struct<(ui256, i1), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK:   %[[P1:.*]] = sol.gep %[[E]], %[[I1]] : !sol.struct<(ui256, i1), Storage>, ui64, !sol.ptr<i1, Storage>
// CHECK:   %[[V1:.*]] = sol.load %[[P1]] : !sol.ptr<i1, Storage>, i1
// CHECK:   sol.return %[[V0]], %[[V1]] : ui256, i1

// CHECK: sol.state_var @{{.*informationMap.*}} slot 0 offset 0 : !sol.mapping<ui256, !sol.struct<(ui256, !sol.address), Storage>>

// CHECK: sol.func @{{.*informationMap.*}}(%arg0: ui256) -> (ui256, !sol.address) attributes {{.*}}selector = -2144009668 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*informationMap.*}} : !sol.mapping<ui256, !sol.struct<(ui256, !sol.address), Storage>>
// CHECK:   %[[S:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<ui256, !sol.struct<(ui256, !sol.address), Storage>>, ui256, !sol.struct<(ui256, !sol.address), Storage>
// CHECK:   %[[I0:.*]] = sol.constant 0 : ui64
// CHECK:   %[[P0:.*]] = sol.gep %[[S]], %[[I0]] : !sol.struct<(ui256, !sol.address), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   %[[V0:.*]] = sol.load %[[P0]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[I1:.*]] = sol.constant 1 : ui64
// CHECK:   %[[P1:.*]] = sol.gep %[[S]], %[[I1]] : !sol.struct<(ui256, !sol.address), Storage>, ui64, !sol.ptr<!sol.address, Storage>
// CHECK:   %[[V1:.*]] = sol.load %[[P1]] : !sol.ptr<!sol.address, Storage>, !sol.address
// CHECK:   sol.return %[[V0]], %[[V1]] : ui256, !sol.address

contract Flat {
    struct Point {
        uint256 x;
        uint256 y;
    }

    Point public origin;
}

contract Gapped {
    struct S {
        uint256 a;
        mapping(uint256 => uint256) m;
        uint256[] array;
        address b;
    }

    S public s;
}

contract Labeled {
    struct S {
        uint256 a;
        string label;
    }

    S public s;
}

contract Nested {
    struct I {
        uint256 v;
    }

    struct S {
        uint256 a;
        I i;
    }

    S public s;
}

contract NestedKeyed {
    struct I {
        uint256 v;
    }

    struct S {
        I i;
    }

    S[] public array;
}

contract Records {
    struct Item {
        uint256 id;
        bool ok;
    }

    Item[] public items;
}

contract Registry {
    struct Information {
        uint256 a;
        address b;
    }

    mapping(uint256 => Information) public informationMap;
}
