// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.state_var @x_{{.*}} slot 0 offset 0 : ui256
// CHECK: sol.state_var @s_{{.*}} slot 1 offset 0 : !sol.string<Storage>
// CHECK: sol.state_var @small_{{.*}} slot 2 offset 0 : ui8
// CHECK: sol.state_var @flag_{{.*}} slot 2 offset 1 : i1

// CHECK: sol.func @{{.*constructor.*}}
// CHECK:   sol.constant 42
// CHECK:   sol.addr_of @x_{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.string_lit "hello" -> !sol.string<Memory>
// CHECK:   sol.addr_of @s_{{.*}} : !sol.string<Storage>
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Storage>
// CHECK:   sol.return

contract C {
    uint256 x = 42;
    string s = "hello";
    uint8 small;
    bool flag;
}
