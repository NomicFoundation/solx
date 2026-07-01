// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.malloc :{{ +}}!sol.struct<(ui256, !sol.string<Memory>), Memory>
// CHECK: sol.constant 0 : ui64
// CHECK: sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, !sol.string<Memory>), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK: sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Memory>
// CHECK: sol.constant 1 : ui64
// CHECK: sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, !sol.string<Memory>), Memory>, ui64, !sol.ptr<!sol.string<Memory>, Memory>
// CHECK: sol.store %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Memory>

contract C {
    struct S { uint256 a; string b; }

    function build(uint256 x, string memory y) public pure returns (S memory) {
        return S(x, y);
    }
}
