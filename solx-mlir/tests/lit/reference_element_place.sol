// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Indexing an array of a *reference* element type in `CallData` or `Storage`
// exercises `Type::address_type`'s reference-element arm: a reference element in
// Storage/CallData is its own place, so the `sol.gep` that selects the element
// yields the struct type directly (`!sol.struct<…, CallData>` /
// `!sol.struct<…, Storage>`) rather than wrapping it in a `!sol.ptr<…>`. A scalar
// field of that struct is then a `!sol.ptr` place, confirming the distinction.
// Both backends agree on the gep result types; function order differs, so
// CHECK-DAG is used.

contract C {
    struct S { uint256 a; uint256 b; }
    S[] sArr;

    function calldataStructArr(S[] calldata ss, uint256 i) external pure returns (uint256) {
        return ss[i].a;
    }

    function storageStructElem(uint256 i) external view returns (uint256) {
        S storage s = sArr[i];
        return s.a;
    }
}

// CHECK-DAG: sol.func @{{.*calldataStructArr.*}}
// CHECK-DAG:   %{{.*}} = sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.struct<(ui256, ui256), CallData>, CallData>, ui256, !sol.struct<(ui256, ui256), CallData>

// CHECK-DAG: sol.func @{{.*storageStructElem.*}}
// CHECK-DAG:   %{{.*}} = sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.struct<(ui256, ui256), Storage>, Storage>, ui256, !sol.struct<(ui256, ui256), Storage>
