// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `blockhash(n)` lowers to `sol.blockhash` over the `ui256` block number,
// yielding a `fixedbytes<32>`.

// CHECK: sol.blockhash %{{.*}} : <32>

contract C {
    function bh(uint256 n) public view returns (bytes32) {
        return blockhash(n);
    }
}
