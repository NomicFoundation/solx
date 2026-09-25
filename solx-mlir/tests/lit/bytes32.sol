// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func {{.*}}fixed_bytes{{.*}}!sol.fixedbytes<32>{{.*}}!sol.fixedbytes<32>

contract C {
    function fixed_bytes(bytes32 v) public pure returns (bytes32) { return v; }
}
