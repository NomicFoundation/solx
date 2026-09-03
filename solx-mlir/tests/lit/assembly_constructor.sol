// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Assembly in a constructor lowers into the constructor's own `sol.inline_asm`, so a store
// there runs in the deploy code.
// CHECK: sol.func @{{.*}} attributes {{{.*}}kind = #Constructor
// CHECK:   sol.inline_asm {
// CHECK:     yul.constant 0
// CHECK:     yul.sstore

contract C {
    uint256 slot0;

    constructor(uint256 x) {
        assembly {
            sstore(0, x)
        }
    }
}
