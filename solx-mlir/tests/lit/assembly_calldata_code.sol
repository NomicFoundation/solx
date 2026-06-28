// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}
// CHECK: yul.calldataload
// CHECK: yul.calldatasize
// CHECK: yul.calldatacopy
// CHECK: yul.codesize
// CHECK: yul.codecopy
// CHECK: yul.returndatasize
// CHECK: yul.returndatacopy
// CHECK: yul.keccak256

contract C {
    function f() public returns (uint256 r) {
        assembly {
            r := calldataload(0)
            r := calldatasize()
            calldatacopy(0, 0, 32)
            r := codesize()
            codecopy(0, 0, 32)
            r := returndatasize()
            returndatacopy(0, 0, 32)
            r := keccak256(0, 32)
        }
    }
}
