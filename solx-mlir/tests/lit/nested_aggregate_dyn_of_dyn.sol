// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// uint256[][] dynamic-of-dynamic. Indexing the outer array is a sol.gep that
// yields a ptr to an inner !sol.array, which is loaded; a second sol.gep then
// indexes the inner array for the element, while .length applies sol.length to
// the loaded inner array. Both backends emit byte-identical op chains; only the
// function emission order differs (solx alphabetical: innerLen, readNested;
// solc source order: readNested, innerLen), so the bodies are pinned under
// split prefixes.

// CHECK-SOLX: sol.func {{.*}}innerLen{{.*}}-> ui256
// CHECK-SOLX:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Memory>, Memory>, ui256, !sol.ptr<!sol.array<? x ui256, Memory>, Memory>
// CHECK-SOLX:   sol.load %{{.*}} : !sol.ptr<!sol.array<? x ui256, Memory>, Memory>, !sol.array<? x ui256, Memory>
// CHECK-SOLX:   sol.length %{{.*}} : !sol.array<? x ui256, Memory>
// CHECK-SOLX: sol.func {{.*}}readNested{{.*}}-> ui256
// CHECK-SOLX:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Memory>, Memory>, ui256, !sol.ptr<!sol.array<? x ui256, Memory>, Memory>
// CHECK-SOLX:   sol.load %{{.*}} : !sol.ptr<!sol.array<? x ui256, Memory>, Memory>, !sol.array<? x ui256, Memory>
// CHECK-SOLX:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>, ui256, !sol.ptr<ui256, Memory>
// CHECK-SOLX:   sol.load %{{.*}} : !sol.ptr<ui256, Memory>, ui256

// CHECK-SOLC: sol.func {{.*}}readNested{{.*}}-> ui256
// CHECK-SOLC:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Memory>, Memory>, ui256, !sol.ptr<!sol.array<? x ui256, Memory>, Memory>
// CHECK-SOLC:   sol.load %{{.*}} : !sol.ptr<!sol.array<? x ui256, Memory>, Memory>, !sol.array<? x ui256, Memory>
// CHECK-SOLC:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>, ui256, !sol.ptr<ui256, Memory>
// CHECK-SOLC:   sol.load %{{.*}} : !sol.ptr<ui256, Memory>, ui256
// CHECK-SOLC: sol.func {{.*}}innerLen{{.*}}-> ui256
// CHECK-SOLC:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x !sol.array<? x ui256, Memory>, Memory>, ui256, !sol.ptr<!sol.array<? x ui256, Memory>, Memory>
// CHECK-SOLC:   sol.load %{{.*}} : !sol.ptr<!sol.array<? x ui256, Memory>, Memory>, !sol.array<? x ui256, Memory>
// CHECK-SOLC:   sol.length %{{.*}} : !sol.array<? x ui256, Memory>

contract C {
    function readNested(uint256[][] memory a, uint256 i, uint256 j) public pure returns (uint256) {
        return a[i][j];
    }

    function innerLen(uint256[][] memory a, uint256 i) public pure returns (uint256) {
        return a[i].length;
    }
}
