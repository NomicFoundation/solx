// RUN: solx --evm-version cancun --emit-mlir=sol %s | FileCheck %s
// RUN: solc --evm-version cancun --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*blobBaseFee.*}}
// CHECK:   yul.blobbasefee

// CHECK: sol.func @{{.*callCode.*}}
// CHECK:   yul.callcode {{.*}}

// CHECK: sol.func @{{.*destroy.*}}
// CHECK:   yul.selfdestruct {{.*}}

// CHECK: sol.func @{{.*externalCodeCopy.*}}
// CHECK:   yul.extcodecopy {{.*}}

// CHECK: sol.func @{{.*memorySize.*}}
// CHECK:   yul.msize

// CHECK: sol.func @{{.*versionedHash.*}}
// CHECK:   yul.blobhash {{.*}}

contract Y {
    function blobBaseFee() external view returns (uint256 a) {
        assembly { a := blobbasefee() }
    }

    function callCode(address x) external returns (uint256 r) {
        assembly { r := callcode(gas(), x, 0, 0, 0, 0, 0) }
    }

    function destroy(address x) external {
        assembly { selfdestruct(x) }
    }

    function externalCodeCopy(address x) external view {
        assembly { extcodecopy(x, 0, 0, 32) }
    }

    function memorySize() external pure returns (uint256 a) {
        assembly { a := msize() }
    }

    function versionedHash() external view returns (uint256 a) {
        assembly { a := blobhash(0) }
    }
}
