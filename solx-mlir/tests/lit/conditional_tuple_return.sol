// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `return cond ? (a, b) : (c, d)` in a multi-value return position lowers to one
// `sol.alloca` slot per result, a `sol.if` whose two branches store the tuple
// elements into those slots, and loads after the join feeding a multi-operand
// `sol.return`.

// CHECK: sol.alloca : !sol.ptr<ui8, Stack>
// CHECK: sol.alloca : !sol.ptr<ui8, Stack>
// CHECK: sol.if
// CHECK: sol.store %{{.*}} : ui8, !sol.ptr<ui8, Stack>
// CHECK: sol.yield
// CHECK: sol.store %{{.*}} : ui8, !sol.ptr<ui8, Stack>
// CHECK: sol.yield
// CHECK: sol.return %{{[0-9]+}}, %{{[0-9]+}} : ui256, ui256

contract C {
    function f(bool cond) public pure returns (uint, uint) {
        return cond ? (1, 2) : (3, 4);
    }
}
