// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Compound assignment on SIGNED `int256` locals: the shared binary-operation
// emitter selects the signed op (`sol.cdiv` for `/=`, `sol.mod` for `%=`) on the
// si256-typed operands. Both backends produce the same op set; only the order of
// the two operand loads differs (CHECK-DAG). Functions are checked in alphabetical
// order (f, g), matching solx's walk and the source order solc uses.

// CHECK: sol.func @{{.*f.*}}(%arg0: si256, %arg1: si256) -> si256
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<si256, Stack>, si256
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<si256, Stack>, si256
// CHECK:   sol.cdiv %{{.*}}, %{{.*}} : si256
// CHECK:   sol.store %{{.*}}, %{{.*}} : si256, !sol.ptr<si256, Stack>

// CHECK: sol.func @{{.*g.*}}(%arg0: si256, %arg1: si256) -> si256
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<si256, Stack>, si256
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<si256, Stack>, si256
// CHECK:   sol.mod %{{.*}}, %{{.*}} : si256
// CHECK:   sol.store %{{.*}}, %{{.*}} : si256, !sol.ptr<si256, Stack>

contract C {
    function f(int256 x, int256 y) public pure returns (int256) { x /= y; return x; }
    function g(int256 x, int256 y) public pure returns (int256) { x %= y; return x; }
}
