// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*assign_fixed.*}}
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.array<3 x ui256, Memory>, !sol.array<3 x ui256, Storage>

// CHECK: sol.func @{{.*assign_dynamic.*}}
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.array<3 x ui8, Memory>, !sol.array<? x ui256, Storage>

// CHECK: sol.func @{{.*assign_string.*}}
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Storage>

// CHECK: sol.func @{{.*assign_struct.*}}
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Memory>, !sol.struct<(ui256, ui256), Storage>

// CHECK: sol.func @{{.*rebind.*}}
// CHECK:   sol.length %{{.*}} : !sol.array<? x ui256, Storage>

// CHECK: sol.func @{{.*from_calldata.*}}
// CHECK:   sol.data_loc_cast %{{.*}} : !sol.array<? x ui256, CallData>, !sol.array<? x ui256, Memory>

// CHECK: sol.func @{{.*from_storage.*}}
// CHECK:   sol.data_loc_cast %{{.*}} : !sol.array<? x ui256, Storage>, !sol.array<? x ui256, Memory>

contract C {
    uint256[3] fixed_array;
    uint256[] dynamic_array;
    string text;

    struct S {
        uint256 a;
        uint256 b;
    }

    S s;

    function assign_fixed(uint256 value) public {
        fixed_array = [value, 2, 3];
    }

    function assign_dynamic() public {
        dynamic_array = [1, 2, 3];
    }

    function assign_string() public {
        text = "abc";
    }

    function assign_struct(uint256 value) public {
        s = S(value, 2);
    }

    function rebind() public view returns (uint256) {
        uint256[] storage pointer = dynamic_array;
        return pointer.length;
    }

    function from_calldata(uint256[] calldata source) external pure returns (uint256) {
        uint256[] memory copy = source;
        return copy.length;
    }

    function from_storage() public view returns (uint256) {
        uint256[] memory copy = dynamic_array;
        return copy.length;
    }
}
