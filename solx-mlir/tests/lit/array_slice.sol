// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}bounded{{.*}}-> !sol.array<? x ui256, CallData>
// CHECK:   sol.slice %{{.*}}[%{{.*}} : %{{.*}}] : !sol.array<? x ui256, CallData>, ui256, ui256 -> !sol.array<? x ui256, CallData>

// CHECK: sol.func {{.*}}from_start{{.*}}-> !sol.array<? x ui256, CallData>
// CHECK:   sol.constant 0 : ui256
// CHECK:   sol.slice %{{.*}}[%{{.*}} : %{{.*}}] : !sol.array<? x ui256, CallData>, ui256, ui256 -> !sol.array<? x ui256, CallData>

// CHECK: sol.func {{.*}}to_end{{.*}}-> !sol.array<? x ui256, CallData>
// CHECK:   sol.length %{{.*}} : !sol.array<? x ui256, CallData>
// CHECK:   sol.slice %{{.*}}[%{{.*}} : %{{.*}}] : !sol.array<? x ui256, CallData>, ui256, ui256 -> !sol.array<? x ui256, CallData>

// CHECK: sol.func {{.*}}full_open{{.*}}-> !sol.array<? x ui256, CallData>
// CHECK:   sol.constant 0 : ui256
// CHECK:   sol.length %{{.*}} : !sol.array<? x ui256, CallData>
// CHECK:   sol.slice %{{.*}}[%{{.*}} : %{{.*}}] : !sol.array<? x ui256, CallData>, ui256, ui256 -> !sol.array<? x ui256, CallData>

// CHECK: sol.func {{.*}}of_bytes{{.*}}-> !sol.string<CallData>
// CHECK:   sol.slice %{{.*}}[%{{.*}} : %{{.*}}] : !sol.string<CallData>, ui256, ui256 -> !sol.string<CallData>

contract ArraySlice {
    function bounded(uint256[] calldata array, uint256 start, uint256 end)
        external
        pure
        returns (uint256[] calldata)
    {
        return array[start:end];
    }

    function from_start(uint256[] calldata array, uint256 end)
        external
        pure
        returns (uint256[] calldata)
    {
        return array[:end];
    }

    function to_end(uint256[] calldata array, uint256 start)
        external
        pure
        returns (uint256[] calldata)
    {
        return array[start:];
    }

    function full_open(uint256[] calldata array)
        external
        pure
        returns (uint256[] calldata)
    {
        return array[:];
    }

    function of_bytes(bytes calldata data, uint256 start, uint256 end)
        external
        pure
        returns (bytes calldata)
    {
        return data[start:end];
    }
}
