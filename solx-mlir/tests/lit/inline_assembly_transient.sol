// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}f
// CHECK: yul.tstore
// CHECK: yul.tload
// CHECK: yul.mcopy

contract C {
    uint256 t;

    function f(uint256 x) public returns (uint256 r) {
        assembly {
            tstore(t.slot, x)
            r := tload(t.slot)
            mcopy(0, 32, 64)
        }
    }
}
