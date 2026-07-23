// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*local_array.*}}
// CHECK:   sol.malloc zero_init : !sol.array<? x ui256, Memory>

// CHECK: sol.func @{{.*local_struct.*}}
// CHECK:   sol.malloc zero_init : !sol.struct<(ui256, ui256), Memory>

// CHECK: sol.func @{{.*local_string.*}}
// CHECK:   sol.malloc : !sol.string<Memory>

// CHECK: sol.func @{{.*named_array.*}}
// CHECK:   sol.malloc zero_init : !sol.array<? x ui256, Memory>

// CHECK: sol.func @{{.*unnamed_array.*}}
// CHECK:   sol.alloca : !sol.ptr<!sol.array<? x ui256, Memory>, Stack>
// CHECK:   sol.malloc zero_init : !sol.array<? x ui256, Memory>
// CHECK:   sol.return %{{.*}} : !sol.array<? x ui256, Memory>

// CHECK: sol.func @{{.*local_fixed_array.*}}
// CHECK:   sol.malloc zero_init : !sol.array<3 x ui256, Memory>

// CHECK: sol.func @{{.*local_bytes.*}}
// CHECK:   sol.malloc : !sol.string<Memory>

// CHECK: sol.func @{{.*local_struct_with_array.*}}
// CHECK:   sol.malloc zero_init : !sol.struct<(ui256, !sol.array<? x ui256, Memory>), Memory>

// CHECK: sol.func @{{.*named_storage.*}}
// CHECK:   sol.default_storage : !sol.array<? x ui256, Storage>

// CHECK: sol.func @{{.*named_calldata.*}}
// CHECK:   sol.default_calldata : !sol.array<? x ui256, CallData>

contract C {
    struct S {
        uint256 a;
        uint256 b;
    }

    struct N {
        uint256 x;
        uint256[] a;
    }

    uint256[] stored;

    function local_array() public pure returns (uint256[] memory) {
        uint256[] memory a;
        return a;
    }

    function local_struct() public pure returns (S memory) {
        S memory s;
        return s;
    }

    function local_string() public pure returns (string memory) {
        string memory t;
        return t;
    }

    function named_array() public pure returns (uint256[] memory a) {}

    function unnamed_array() public pure returns (uint256[] memory) {}

    function local_fixed_array() public pure returns (uint256[3] memory) {
        uint256[3] memory a;
        return a;
    }

    function local_bytes() public pure returns (bytes memory) {
        bytes memory b;
        return b;
    }

    function local_struct_with_array() public pure returns (N memory) {
        N memory n;
        return n;
    }

    function named_storage() internal view returns (uint256[] storage r) {
        r = stored;
    }

    function named_calldata(uint256[] calldata a) external pure returns (uint256[] calldata r) {
        r = a;
    }
}
