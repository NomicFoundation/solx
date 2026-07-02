// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}
// CHECK: yul.mcopy
// CHECK: yul.tstore
// CHECK: yul.tload
// CHECK: yul.blockhash
// CHECK: yul.prevrandao

contract C {
    function f(uint256 v) public returns (uint256 r) {
        assembly {
            mcopy(0, 32, 64)
            tstore(0, v)
            r := tload(0)
            r := blockhash(1)
            r := prevrandao()
        }
    }
}
