// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.ripemd160{{.*}}: (!sol.string<Memory>) -> !sol.fixedbytes<20>

contract C {
    function digest(bytes memory data) public pure returns (bytes20) { return ripemd160(data); }
}
