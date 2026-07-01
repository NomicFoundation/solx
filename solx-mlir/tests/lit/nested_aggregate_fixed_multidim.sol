// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}read{{.*}}!sol.array<2 x !sol.array<3 x ui256, Memory>, Memory>{{.*}}-> ui256
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<2 x !sol.array<3 x ui256, Memory>, Memory>, ui256, !sol.ptr<!sol.array<3 x ui256, Memory>, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.array<3 x ui256, Memory>, Memory>, !sol.array<3 x ui256, Memory>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<3 x ui256, Memory>, ui256, !sol.ptr<ui256, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Memory>, ui256

contract C {
    function read(uint256[3][2] memory a, uint256 i, uint256 j) public pure returns (uint256) {
        return a[i][j];
    }
}
