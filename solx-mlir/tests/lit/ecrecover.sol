// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.ecrecover{{.*}}: (!sol.fixedbytes<32>, ui8, !sol.fixedbytes<32>, !sol.fixedbytes<32>) -> !sol.address

contract C {
    function recover(bytes32 h, uint8 v, bytes32 r, bytes32 s) public pure returns (address) {
        return ecrecover(h, v, r, s);
    }
}
