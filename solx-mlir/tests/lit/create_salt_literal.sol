// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `new Child{salt: hex"01"}()` supplies the CREATE2 salt as a string literal.
// The `{salt: …}` capture emits the literal toward `bytes32` so it folds to a
// fixedbytes constant (not a memory string), which is then widened back to
// `ui256` and threaded into `sol.new` as the CREATE2 salt operand. solx-only:
// solc's MLIR frontend mis-lowers the literal salt (it leaves a
// `!sol.string<Memory>` operand feeding `sol.cast`, failing module
// verification), so there is no `solc` RUN line.

// CHECK: %[[B:.*]] = sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK: %[[S:.*]] = sol.bytes_cast %[[B]] : !sol.fixedbytes<32> to ui256
// CHECK: sol.new "Child" value = %{{.*}} salt = %[[S]] ctor() : !sol.contract<"Child">

contract Child {}

contract C {
    function f() external returns (Child) {
        return new Child{salt: hex"01"}();
    }
}
