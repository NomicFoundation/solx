// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s
// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=SOLX

// CHECK: sol.inline_asm {
// CHECK:   yul.mul %{{.*}}, %c2_i256

// solx emits the contract module first and lands the copy at its first reference, print-init
// emits the library module first, so the RUN line above cannot check the framing.
// SOLX: sol.contract @{{.*}}C{{.*}} {
// SOLX:   sol.func @{{.*use.*}}
// SOLX:   sol.func private @{{.*twice.*}}
// SOLX:     sol.inline_asm {
// SOLX:       yul.mul %{{.*}}, %c2_i256
// SOLX: } {kind = #Contract}
// SOLX: sol.contract @{{.*}}L{{.*}} {
// SOLX:   sol.func private @{{.*twice.*}}
// SOLX:     sol.inline_asm {
// SOLX:       yul.mul %{{.*}}, %c2_i256
// SOLX: } {kind = #Library}

library L {
    function twice(uint256 x) internal pure returns (uint256 r) {
        assembly {
            r := mul(x, 2)
        }
    }
}

contract C {
    function use(uint256 x) public pure returns (uint256 r) {
        r = L.twice(x);
    }
}
