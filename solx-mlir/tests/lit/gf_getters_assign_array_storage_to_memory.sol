// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Assigning a storage array reference to a MEMORY local (`dst = src`): the storage
// reference is bridged to the memory ABI with `sol.data_loc_cast` Storage ->
// Memory, then stored into the local's stack slot. Both backends emit the same
// addr_of / data_loc_cast / store for the assignment itself. (Only the preceding
// `new uint256[](2)` allocation differs in how the size constant is typed, which
// is not part of the assignment under test, so checks start at the addr_of.)

// CHECK: sol.func @{{.*f.*}}()
// CHECK:   %[[A:.*]] = sol.addr_of @{{.*src.*}} : !sol.array<? x ui256, Storage>
// CHECK:   %[[C:.*]] = sol.data_loc_cast %[[A]] : !sol.array<? x ui256, Storage>, !sol.array<? x ui256, Memory>
// CHECK:   sol.store %[[C]], %{{.*}} : !sol.array<? x ui256, Memory>, !sol.ptr<!sol.array<? x ui256, Memory>, Stack>

contract C {
    uint256[] src;
    function f() public {
        uint256[] memory dst = new uint256[](2);
        dst = src;
    }
}
