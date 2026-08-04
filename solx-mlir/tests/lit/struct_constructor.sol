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

// CHECK: sol.func {{.*}}build_named{{.*}}-> !sol.struct<(ui256, ui256), Memory>
// CHECK:   %[[STRUCT:.*]] = sol.malloc :{{ +}}!sol.struct<(ui256, ui256), Memory>
// CHECK:   %[[FIRST:.*]] = sol.constant 0 : ui64
// CHECK:   %[[A:.*]] = sol.gep %[[STRUCT]], %[[FIRST]] : !sol.struct<(ui256, ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK:   %[[ONE:.*]] = sol.constant 1 : ui8
// CHECK:   %[[A_VALUE:.*]] = sol.cast %[[ONE]] : ui8 to ui256
// CHECK:   sol.store %[[A_VALUE]], %[[A]] : ui256, !sol.ptr<ui256, Memory>
// CHECK:   %[[SECOND:.*]] = sol.constant 1 : ui64
// CHECK:   %[[B:.*]] = sol.gep %[[STRUCT]], %[[SECOND]] : !sol.struct<(ui256, ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK:   %[[TWO:.*]] = sol.constant 2 : ui8
// CHECK:   %[[B_VALUE:.*]] = sol.cast %[[TWO]] : ui8 to ui256
// CHECK:   sol.store %[[B_VALUE]], %[[B]] : ui256, !sol.ptr<ui256, Memory>

// CHECK: sol.func {{.*}}build_tagged{{.*}}-> !sol.struct<(!sol.fixedbytes<4>, ui256), Memory>
// CHECK:   sol.constant {{.*}} : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

contract C {
    struct S { uint256 a; uint256 b; }

    struct T { bytes4 tag; uint256 n; }

    function build(uint256 x, uint256 y) public pure returns (S memory) {
        return S(x, y);
    }

    function build_named() public pure returns (S memory) {
        return S({b: 2, a: 1});
    }

    function build_tagged() public pure returns (T memory) {
        return T("abcd", 7);
    }
}
