// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}bytes_id{{.*}}!sol.string<Memory>{{.*}}!sol.string<Memory>

contract C {
    function bytes_id(bytes memory b) public pure returns (bytes memory) { return b; }
}
