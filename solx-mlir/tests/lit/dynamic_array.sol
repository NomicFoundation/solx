// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}dyn_array{{.*}}!sol.array<{{.+}} x ui256, Memory>

contract C {
    function dyn_array(uint256[] memory a) public pure returns (uint256[] memory) { return a; }
}
