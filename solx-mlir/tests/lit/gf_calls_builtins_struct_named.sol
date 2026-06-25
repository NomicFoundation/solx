// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A struct constructor with named arguments (`S({b: y, a: x})`) reorders the
// initialisers into member-declaration order, allocates the struct, and stores
// each field at its declared `gep` index — identical to a positional `S(x, y)`.

// CHECK: sol.malloc :{{ +}}!sol.struct<(ui256, ui256), Memory>
// CHECK: sol.constant 0 : ui64
// CHECK: sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK: sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Memory>
// CHECK: sol.constant 1 : ui64
// CHECK: sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK: sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Memory>

contract C {
    struct S { uint256 a; uint256 b; }

    function build(uint256 x, uint256 y) public pure returns (S memory) {
        return S({b: y, a: x});
    }
}
