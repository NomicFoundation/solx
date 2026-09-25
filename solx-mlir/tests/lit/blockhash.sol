// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*bh.*}}(%{{.*}}: ui256) -> !sol.fixedbytes<32>
// CHECK:   sol.blockhash %{{.*}} : <32>

contract C {
    function bh(uint256 n) public view returns (bytes32) {
        return blockhash(n);
    }
}
