// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*constructor.*}}
// CHECK:   sol.constant 42
// CHECK:   sol.addr_of @{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.string_lit "hello" -> !sol.string<Memory>
// CHECK:   sol.addr_of @{{.*}} : !sol.string<Storage>
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Storage>
// CHECK:   sol.return

contract C {
    uint256 x = 42;
    string s = "hello";
}
