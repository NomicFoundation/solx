// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A library external function selects on solc's `signatureInExternalFunction(structsByName = true)`:
// a struct parameter is named by its scope-qualified name (`L.S`, `I.S`) rather than its ABI tuple
// `(uint256...)`, and a `storage` parameter keeps its data-location. Slang's `compute_selector` hashes
// the plain ABI canonical signature, so the selector is recomputed (see `library_aware_signature`):
//   f(L.S storage) -> 0xc1d97e2c = -1043495828    g(L.S) -> 0x4221b079 = 1109318265
// The two `a` overloads take structs from different scopes: `I.S` (1 field, tuple `(uint256)`) and the
// library's own `S` (2 fields, tuple `(uint256,uint256)`). Their ABI tuples already differ, so this
// pins the scope-qualified selector VALUES against solc (`a(I.S)` -> -1327751287, `a(L.S)` ->
// -968416976) — i.e. that the qualifier is `I.S`/`L.S`, not the bare names or the tuples.

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
