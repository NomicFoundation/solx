// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Signed / width-aware arithmetic Yul opcodes lower to the matching Yul-dialect
// ops (rule 16: Yul never crosses into the Sol dialect). Covers the opcodes not
// already exercised by assembly_arithmetic.sol: sdiv, smod, sar, byte,
// signextend, and the ternary modular ops addmod/mulmod.

// CHECK: sol.func @{{.*f.*}}
// CHECK: yul.sdiv
// CHECK: yul.smod
// CHECK: yul.sar
// CHECK: yul.byte
// CHECK: yul.signextend
// CHECK: yul.addmod
// CHECK: yul.mulmod

contract C {
    function f(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := sdiv(a, b)
            r := smod(a, b)
            r := sar(a, r)
            r := byte(a, r)
            r := signextend(a, r)
            r := addmod(a, b, r)
            r := mulmod(a, b, r)
        }
    }
}
