// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}(%arg0: ui256, %arg1: ui256) -> ui256
// CHECK: %[[MAP:.*]] = sol.map {{.*}} : !sol.mapping<ui256, !sol.array<? x ui256, Storage>>, ui256, !sol.array<? x ui256, Storage>
// CHECK: %[[ELT:.*]] = sol.gep %[[MAP]], {{.*}} : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK: sol.load %[[ELT]] : !sol.ptr<ui256, Storage>, ui256

contract C {
    mapping(uint256 => uint256[]) m;

    function f(uint256 i, uint256 j) external view returns (uint256) {
        return m[i][j];
    }
}
