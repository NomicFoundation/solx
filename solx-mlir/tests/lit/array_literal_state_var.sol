// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*}} slot 0 offset 0 : !sol.array<3 x ui256, Storage>

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Constructor
// CHECK: sol.addr_of @{{.*}} : !sol.array<3 x ui256, Storage>
// CHECK: sol.array_lit %{{.*}}, %{{.*}}, %{{.*}} : (ui8, ui8, ui8) -> !sol.array<3 x ui8, Memory>
// CHECK: sol.copy %{{.*}}, %{{.*}} : !sol.array<3 x ui8, Memory>, !sol.array<3 x ui256, Storage>

contract ArrayLiteralStateVariable {
    uint256[3] data = [1, 2, 3];

    function first() public view returns (uint256) {
        return data[0];
    }
}
