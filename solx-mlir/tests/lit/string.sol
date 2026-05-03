// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}string_id{{.*}}!sol.string<Memory>{{.*}}!sol.string<Memory>

contract C {
    function string_id(string memory s) public pure returns (string memory) { return s; }
}
