// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.alloca : !sol.ptr<ui8, Stack>
// CHECK: sol.alloca : !sol.ptr<ui8, Stack>
// CHECK: sol.if
// CHECK: sol.store %{{.*}} : ui8, !sol.ptr<ui8, Stack>
// CHECK: sol.yield
// CHECK: sol.store %{{.*}} : ui8, !sol.ptr<ui8, Stack>
// CHECK: sol.yield
// CHECK: sol.return %{{[0-9]+}}, %{{[0-9]+}} : ui256, ui256

contract C {
    function f(bool condition) public pure returns (uint, uint) {
        return condition ? (1, 2) : (3, 4);
    }
}
