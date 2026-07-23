// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}build{{.*}}-> !sol.struct<(ui256, ui256), Memory>
// CHECK:   sol.malloc :{{ +}}!sol.struct<(ui256, ui256), Memory>
// CHECK:   sol.constant 0 : ui64
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Memory>
// CHECK:   sol.constant 1 : ui64
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Memory>

// CHECK: sol.func {{.*}}build_tagged{{.*}}-> !sol.struct<(!sol.fixedbytes<4>, ui256), Memory>
// CHECK:   sol.constant {{.*}} : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

contract C {
    struct S { uint256 a; uint256 b; }

    struct T { bytes4 tag; uint256 n; }

    function build(uint256 x, uint256 y) public pure returns (S memory) {
        return S(x, y);
    }

    function build_tagged() public pure returns (T memory) {
        return T("abcd", 7);
    }
}
