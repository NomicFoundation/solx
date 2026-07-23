// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @x_{{[0-9]+}} slot 0 offset 0 : ui256
// CHECK: sol.state_var @s_{{[0-9]+}} slot 1 offset 0 : !sol.string<Storage>
// CHECK: sol.state_var @small_{{[0-9]+}} slot 2 offset 0 : ui8
// CHECK: sol.state_var @flag_{{[0-9]+}} slot 2 offset 1 : i1
// CHECK: sol.state_var @data_{{[0-9]+}} slot 3 offset 0 : !sol.array<3 x ui256, Storage>

// CHECK: sol.func @{{.*}} attributes {kind = #Constructor
// CHECK:   sol.addr_of @x_{{[0-9]+}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.constant 42
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.addr_of @s_{{[0-9]+}} : !sol.string<Storage>
// CHECK:   sol.string_lit "hello" -> !sol.string<Memory>
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Storage>
// CHECK:   sol.addr_of @data_{{[0-9]+}} : !sol.array<3 x ui256, Storage>
// CHECK:   sol.array_lit %{{.*}}, %{{.*}}, %{{.*}} : (ui8, ui8, ui8) -> !sol.array<3 x ui8, Memory>
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.array<3 x ui8, Memory>, !sol.array<3 x ui256, Storage>
// CHECK:   sol.return

contract C {
    uint256 x = 42;
    string s = "hello";
    uint8 small;
    bool flag;
    uint256[3] data = [1, 2, 3];
}
