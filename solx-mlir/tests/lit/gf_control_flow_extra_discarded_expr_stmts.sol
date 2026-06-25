// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Expression statements whose value is discarded: postfix/prefix ++/-- in
// statement position lower to cadd/csub + store, and a bare `a + a;` / `a;`
// still emit the computation (result unused) before the trailing return.

// CHECK: sol.func @{{.*stmts.*}}
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.csub %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.csub %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.return

contract C {
    uint256 public x;
    function stmts(uint256 a) public {
        x++;
        x--;
        ++x;
        --x;
        a + a;
        a;
    }
}
