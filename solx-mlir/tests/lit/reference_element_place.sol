// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*calldataStructArray.*}}
// CHECK:   %{{.*}} = sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.struct<(ui256, ui256), CallData>, CallData>, ui256, !sol.struct<(ui256, ui256), CallData>
// CHECK: sol.func @{{.*storageStructElement.*}}
// CHECK:   %{{.*}} = sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.struct<(ui256, ui256), Storage>, Storage>, ui256, !sol.struct<(ui256, ui256), Storage>

contract C {
    struct S { uint256 a; uint256 b; }

    S[] sArray;

    function calldataStructArray(S[] calldata ss, uint256 i) external pure returns (uint256) {
        return ss[i].a;
    }

    function storageStructElement(uint256 i) external view returns (uint256) {
        S storage s = sArray[i];
        return s.a;
    }
}
