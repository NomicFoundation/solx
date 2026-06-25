// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Fixed multi-dimensional uint256[3][2]: Solidity reads the dims outside-in, so
// the parameter type is array<2 x array<3 x ui256>>. a[i] is a sol.gep into the
// outer (size 2) array yielding a ptr to the inner array<3>, loaded, then a[i][j]
// is a second sol.gep into the inner (size 3) array. Both backends agree exactly.

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
