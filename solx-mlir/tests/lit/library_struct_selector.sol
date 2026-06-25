// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A library external function selects on solc's `signatureInExternalFunction(structsByName = true)`:
// a struct parameter is named by its scope-qualified name (`L.S`, `I.S`) rather than its ABI tuple
// `(uint256)`, and a `storage` parameter keeps its data-location. Slang's `compute_selector` hashes
// the plain ABI canonical signature, so the selector is recomputed (see `library_aware_selector`):
//   f(L.S storage) -> 0xc1d97e2c = -1043495828    g(L.S) -> 0x4221b079 = 1109318265
// The two `a` overloads take same-arity structs from different scopes (`I.S` vs `L.S`); they must
// select distinctly (`a(I.S)` -> -1327751287, `a(L.S)` -> -968416976), which the scope qualifier —
// not the ABI tuple, identical for both — is what distinguishes.

pragma abicoder v2;
interface I { struct S { uint256 a; } }
library L {
    struct S { uint256 b; uint256 a; }
    function f(S storage s) external view returns (uint256) { return s.a; }
    function g(S memory m) external pure returns (uint256) { return m.a; }
    function a(I.S memory) external pure returns (uint256) { return 1; }
    function a(S memory) external pure returns (uint256) { return 2; }
}

// CHECK-DAG: sol.func @{{.*}}(%{{.*}}: !sol.struct<(ui256, ui256), Storage>) -> ui256 {{.*}}selector = -1043495828
// CHECK-DAG: sol.func @{{.*}}(%{{.*}}: !sol.struct<(ui256, ui256), Memory>) -> ui256 {{.*}}selector = 1109318265
// CHECK-DAG: sol.func @{{.*}}(%{{.*}}: !sol.struct<(ui256), Memory>) -> ui256 {{.*}}selector = -1327751287
// CHECK-DAG: sol.func @{{.*}}(%{{.*}}: !sol.struct<(ui256, ui256), Memory>) -> ui256 {{.*}}selector = -968416976
