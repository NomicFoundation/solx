// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.keccak256{{.*}}: (!sol.string<Memory>) -> !sol.fixedbytes<32>

contract C {
    function digest(bytes memory data) public pure returns (bytes32) { return keccak256(data); }
}
