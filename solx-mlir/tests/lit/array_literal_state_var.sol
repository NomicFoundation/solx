// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A state variable initialized with an array literal lowers, in the implicit
// constructor, to: addr_of the storage slot, sol.array_lit building the literal
// as a memory array (element type = the literals' common type, here ui8), then
// sol.copy memory->storage. sol.copy performs the per-element widening to the
// declared storage element type (ui8 -> ui256), so no explicit cast is emitted.

// CHECK: sol.state_var @{{.*}} slot 0 offset 0 : !sol.array<3 x ui256, Storage>

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Constructor
// CHECK: sol.addr_of @{{.*}} : !sol.array<3 x ui256, Storage>
// CHECK: sol.array_lit %{{.*}}, %{{.*}}, %{{.*}} : (ui8, ui8, ui8) -> !sol.array<3 x ui8, Memory>
// CHECK: sol.copy %{{.*}}, %{{.*}} : !sol.array<3 x ui8, Memory>, !sol.array<3 x ui256, Storage>

contract ArrayLitStateVar {
    uint256[3] data = [1, 2, 3];

    function first() public view returns (uint256) {
        return data[0];
    }
}
