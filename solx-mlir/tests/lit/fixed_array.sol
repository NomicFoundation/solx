// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}fixed_array{{.*}}!sol.array<4 x ui256, Memory>

contract C {
    function fixed_array(uint256[4] memory a) public pure returns (uint256[4] memory) { return a; }
}
