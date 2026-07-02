// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.blockhash %{{.*}} : <32>

contract C {
    function bh(uint256 n) public view returns (bytes32) {
        return blockhash(n);
    }
}
