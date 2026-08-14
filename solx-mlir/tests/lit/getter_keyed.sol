// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*items.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -1078840878 : i32
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*items.*}} : !sol.array<? x ui256, Storage>
// CHECK:   %[[P:.*]] = sol.gep %[[A]], %arg0 no_panic_bounds : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

// CHECK: sol.func @{{.*fixed_items.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -2078704799 : i32
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*fixed_items.*}} : !sol.array<3 x ui256, Storage>
// CHECK:   %[[P:.*]] = sol.gep %[[A]], %arg0 no_panic_bounds : !sol.array<3 x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

// CHECK: sol.state_var @{{.*balances.*}} slot 0 offset 0 : !sol.mapping<!sol.address, ui256>

// CHECK: sol.func @{{.*balances.*}}(%arg0: !sol.address) -> ui256 attributes {{.*}}selector = 669136355 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*balances.*}} : !sol.mapping<!sol.address, ui256>
// CHECK:   %[[SLOT:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<!sol.address, ui256>, !sol.address, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

// CHECK: sol.state_var @{{.*allowance.*}} slot 0 offset 0 : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>

// CHECK: sol.func @{{.*allowance.*}}(%arg0: !sol.address, %arg1: ui256) -> ui256 attributes {{.*}}selector = -574185103 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*allowance.*}} : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>
// CHECK:   %[[M1:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>, !sol.address, !sol.mapping<ui256, ui256>
// CHECK:   %[[SLOT:.*]] = sol.map %[[M1]], %arg1 : !sol.mapping<ui256, ui256>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

// CHECK: sol.func @{{.*}}tags{{.*}} -> !sol.string<Memory>
// CHECK: sol.func @{{.*}}blobs{{.*}} -> !sol.string<Memory>
// CHECK: sol.data_loc_cast {{.*}} : !sol.string<Storage>, !sol.string<Memory>

// CHECK: sol.func @{{.*scores.*}}(%arg0: !sol.string<Memory>) -> ui256 attributes {{.*}}selector = -846305981 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*scores.*}} : !sol.mapping<!sol.string<Memory>, ui256>
// CHECK:   %[[SLOT:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<!sol.string<Memory>, ui256>, !sol.string<Memory>, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

// CHECK: sol.func @{{.*nested.*}}(%arg0: ui256, %arg1: ui256) -> ui256 attributes {{.*}}selector = 1355935874 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*nested.*}} : !sol.mapping<ui256, !sol.array<? x ui256, Storage>>
// CHECK:   %[[A:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<ui256, !sol.array<? x ui256, Storage>>, ui256, !sol.array<? x ui256, Storage>
// CHECK:   %[[P:.*]] = sol.gep %[[A]], %arg1 no_panic_bounds : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

contract DynamicArray {
    uint256[] public items;
}

contract FixedArray {
    uint256[3] public fixed_items;
}

contract Mapping {
    mapping(address => uint256) public balances;
}

contract MultiKeyMapping {
    mapping(address => mapping(uint256 => uint256)) public allowance;
}

contract ReferenceKeyed {
    string[] public tags;
    mapping(uint256 => bytes) public blobs;
}

contract StringKeyMapping {
    mapping(string => uint256) public scores;
}

contract Table {
    mapping(uint256 => uint256[]) public nested;
}
