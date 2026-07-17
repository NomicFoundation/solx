// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*delete_array.*}}
// CHECK:   sol.delete %{{.*}} : !sol.array<? x ui256, Storage>

// CHECK: sol.func @{{.*delete_scalar.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*delete_bytes.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.fixedbytes<32>, !sol.ptr<!sol.fixedbytes<32>, Storage>

// CHECK: sol.func @{{.*delete_map_entry.*}}
// CHECK:   %[[ENTRY:.*]] = sol.map %{{.*}}, %{{.*}} : !sol.mapping<ui256, ui256>, ui8, !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %[[ENTRY]] : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*delete_struct.*}}
// CHECK:   sol.delete %{{.*}} : !sol.struct<(ui256, ui256), Storage>

// CHECK: sol.func @{{.*delete_struct_field.*}}
// CHECK:   %[[FIELD:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %[[FIELD]] : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*delete_local.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>

contract C {
    uint256 scalar;
    bytes32 word;
    uint256[] array;
    mapping(uint256 => uint256) map;

    struct S {
        uint256 a;
        uint256 b;
    }

    S s;

    function delete_array() public {
        delete array;
    }

    function delete_scalar() public {
        delete scalar;
    }

    function delete_bytes() public {
        delete word;
    }

    function delete_map_entry() public {
        delete map[3];
    }

    function delete_struct() public {
        delete s;
    }

    function delete_struct_field() public {
        delete s.a;
    }

    function delete_local(uint256 value) public pure returns (uint256) {
        delete value;
        return value;
    }
}
