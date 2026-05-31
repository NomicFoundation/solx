// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}f
// CHECK: yul.sstore
// CHECK: sol.add

contract C {
    uint256 s;

    function f(uint256 x) public returns (uint256 r) {
        assembly {
            sstore(s.slot, x)
            r := add(x, 1)
        }
    }
}
