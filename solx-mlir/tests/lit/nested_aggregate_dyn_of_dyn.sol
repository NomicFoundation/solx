// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}innerLength{{.*}}-> ui256
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Memory>, Memory>, ui256, !sol.ptr<!sol.array<? x ui256, Memory>, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.array<? x ui256, Memory>, Memory>, !sol.array<? x ui256, Memory>
// CHECK:   sol.length %{{.*}} : !sol.array<? x ui256, Memory>
// CHECK: sol.func {{.*}}readNested{{.*}}-> ui256
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Memory>, Memory>, ui256, !sol.ptr<!sol.array<? x ui256, Memory>, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.array<? x ui256, Memory>, Memory>, !sol.array<? x ui256, Memory>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>, ui256, !sol.ptr<ui256, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Memory>, ui256

contract C {
    function innerLength(uint256[][] memory a, uint256 i) public pure returns (uint256) {
        return a[i].length;
    }

    function readNested(uint256[][] memory a, uint256 i, uint256 j) public pure returns (uint256) {
        return a[i][j];
    }
}
