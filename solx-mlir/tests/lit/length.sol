// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}length_array{{.*}}!sol.array<? x ui256, Memory>{{.*}}ui256
// CHECK:   sol.length {{.*}} : !sol.array<? x ui256, Memory>
// CHECK: sol.func {{.*}}length_bytes{{.*}}!sol.string<Memory>{{.*}}ui256
// CHECK:   sol.length {{.*}} : !sol.string<Memory>
// CHECK: sol.func {{.*}}length_fixed{{.*}}!sol.array<5 x ui256, Memory>{{.*}}ui256
// CHECK:   sol.length {{.*}} : !sol.array<5 x ui256, Memory>

contract C {
    function length_array(uint256[] memory a) public pure returns (uint256) { return a.length; }

    function length_bytes(bytes memory b) public pure returns (uint256) { return b.length; }

    function length_fixed(uint256[5] memory a) public pure returns (uint256) { return a.length; }
}
