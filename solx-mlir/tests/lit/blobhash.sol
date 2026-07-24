// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*versioned_hash.*}}(%{{.*}}: ui256) -> !sol.fixedbytes<32>
// CHECK:   sol.blobhash %{{.*}} : <32>

contract C {
    function versioned_hash(uint256 index) external view returns (bytes32) {
        return blobhash(index);
    }
}
