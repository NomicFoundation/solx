// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.code {{.*}} : !sol.address -> !sol.string<Memory>

contract C {
    function bytecode(address a) public view returns (bytes memory) { return a.code; }
}
