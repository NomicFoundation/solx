// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG: sol.addmod %{{.*}}, %{{.*}}, %{{.*}} : ui256
// CHECK-DAG: sol.mulmod %{{.*}}, %{{.*}}, %{{.*}} : ui256
// CHECK-DAG: sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK-DAG: sol.ecrecover{{.*}}(!sol.fixedbytes<32>, ui8, !sol.fixedbytes<32>, !sol.fixedbytes<32>) -> !sol.address

contract C {
    function am() public pure returns (uint256) { return addmod(2, 3, 5); }
    function mm() public pure returns (uint256) { return mulmod(2, 3, 5); }
    function ec() public pure returns (address) {
        return ecrecover(bytes32(uint256(1)), 27, bytes32(uint256(2)), bytes32(uint256(3)));
    }
}
