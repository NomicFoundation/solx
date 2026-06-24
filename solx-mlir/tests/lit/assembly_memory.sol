// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Inline-assembly memory and storage opcodes lower to the raw Yul-dialect
// load/store ops (operating on signless `i256`), not the Sol memory ops.

// CHECK: sol.func @{{.*mem.*}}
// CHECK: yul.mstore
// CHECK: yul.mload
// CHECK: yul.mstore8

// CHECK: sol.func @{{.*stor.*}}
// CHECK: yul.sstore
// CHECK: yul.sload

contract C {
    function mem() public pure returns (uint256 r) {
        assembly {
            mstore(0, 42)
            r := mload(0)
            mstore8(32, 7)
        }
    }

    function stor(uint256 v) public returns (uint256 r) {
        assembly {
            sstore(0, v)
            r := sload(0)
        }
    }
}
