// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A low-level `catch (bytes memory data)` binds the whole returndata. It
// populates the `sol.try` fallback region with a `string<Memory>` block
// argument (the raw revert data), alongside the typed Error / Panic regions.
// This is solx-only: solc's MLIR frontend rejects a parameter-bound fallback
// (`SolidityToMLIR.cpp` `!fallbackClause->parameters() && "NYI"`), so there is
// no `solc` RUN line to cross-check — the `sol.try` lowering still handles it.

// CHECK: sol.try
// CHECK: panic {
// CHECK: ^bb0({{.*}}: ui256):
// CHECK: error {
// CHECK: ^bb0({{.*}}: !sol.string<Memory>):
// CHECK: fallback {
// CHECK: ^bb0({{.*}}: !sol.string<Memory>):

interface I {
    function f() external returns (uint256);
}

contract C {
    function g(I i) external returns (uint256) {
        try i.f() returns (uint256 v) {
            return v;
        } catch Error(string memory reason) {
            return bytes(reason).length;
        } catch Panic(uint256 code) {
            return code;
        } catch (bytes memory data) {
            return data.length;
        }
    }
}
