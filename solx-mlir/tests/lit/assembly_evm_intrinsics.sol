// RUN: solx --evm-version cancun --emit-mlir=sol %s | FileCheck %s
// RUN: solc --evm-version cancun --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: yul.msize
// CHECK-DAG: yul.blobbasefee
// CHECK-DAG: yul.blobhash {{.*}}
// CHECK-DAG: yul.extcodecopy {{.*}}
// CHECK-DAG: yul.selfdestruct {{.*}}
// CHECK-DAG: yul.callcode {{.*}}

contract Y {
    function memorySize() external pure returns (uint256 a) {
        assembly { a := msize() }
    }

    function blobBaseFee() external view returns (uint256 a) {
        assembly { a := blobbasefee() }
    }

    function versionedHash() external view returns (uint256 a) {
        assembly { a := blobhash(0) }
    }

    function externalCodeCopy(address x) external view {
        assembly { extcodecopy(x, 0, 0, 32) }
    }

    function destroy(address x) external {
        assembly { selfdestruct(x) }
    }

    function callCode(address x) external returns (uint256 r) {
        assembly { r := callcode(gas(), x, 0, 0, 0, 0, 0) }
    }
}
