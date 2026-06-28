// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// `delete` of a *memory* reference-typed lvalue (a `Pointer` target whose Slang
// type is a reference type, not a storage `sol.delete`). The reset value is a
// freshly default-initialised buffer, re-stored over the local:
//   * a `string` / `bytes` resets to an empty (length-0) buffer;
//   * any other memory aggregate (here a `uint256[]`) resets to a zero-filled
//     default buffer via `sol.malloc zero_init`.
//
// For the array case both compilers emit the same `sol.malloc zero_init` reset.
// For the string/bytes case solx emits a sized, zero-initialised malloc
// (`sol.malloc %c0 zero_init`) while solc's nascent MLIR backend emits a bare
// `sol.malloc`; behavioural parity is covered by the tester, so that op is
// pinned per-tool.

// solx emits the functions alphabetically (delMemArr before delStr); solc
// emits them in source order (delStr before delMemArr).
// CHECK-SOLX: sol.func @{{.*delMemArr.*}}
// CHECK-SOLX: sol.malloc zero_init : {{.*}}!sol.array<? x ui256, Memory>
// CHECK-SOLX: sol.store %{{[0-9]+}}, %{{[0-9]+}} : !sol.array<? x ui256, Memory>
// CHECK-SOLX: sol.func @{{.*delStr.*}}
// CHECK-SOLX: sol.malloc %{{.*}} zero_init : ui256 !sol.string<Memory>
// CHECK-SOLX: sol.store %{{[0-9]+}}, %{{[0-9]+}} : !sol.string<Memory>

// CHECK-SOLC: sol.func @{{.*delStr.*}}
// CHECK-SOLC: sol.malloc : {{.*}}!sol.string<Memory>
// CHECK-SOLC: sol.store %{{[0-9]+}}, %{{[0-9]+}} : !sol.string<Memory>
// CHECK-SOLC: sol.func @{{.*delMemArr.*}}
// CHECK-SOLC: sol.malloc zero_init : {{.*}}!sol.array<? x ui256, Memory>
// CHECK-SOLC: sol.store %{{[0-9]+}}, %{{[0-9]+}} : !sol.array<? x ui256, Memory>
// EXPLAIN: can the two below be just re-ordered?
contract C {
    function delStr() public pure {
        string memory s = "hello";
        delete s;
    }
    function delMemArr() public pure {
        uint256[] memory a = new uint256[](3);
        delete a;
    }
}
