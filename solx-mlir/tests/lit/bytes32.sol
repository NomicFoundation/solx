// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}fixed_bytes{{.*}}!sol.fixedbytes<32>{{.*}}!sol.fixedbytes<32>

contract C {
    function fixed_bytes(bytes32 v) public pure returns (bytes32) { return v; }
}
