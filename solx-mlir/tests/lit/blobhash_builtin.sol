// RUN: solx --evm-version cancun --emit-mlir=sol %s | FileCheck %s
// RUN: solc --evm-version cancun --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.blobhash {{.*}} : <32>

contract C {
    function versionedHash(uint256 index) external view returns (bytes32) {
        return blobhash(index);
    }
}
