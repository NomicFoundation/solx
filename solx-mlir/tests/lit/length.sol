// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func {{.*}}len_bytes{{.*}}!sol.string<Memory>{{.*}}ui256
// CHECK-DAG:   sol.length {{.*}} : !sol.string<Memory>
// CHECK-DAG: sol.func {{.*}}len_arr{{.*}}!sol.array<? x ui256, Memory>{{.*}}ui256
// CHECK-DAG:   sol.length {{.*}} : !sol.array<? x ui256, Memory>
// CHECK-DAG: sol.func {{.*}}len_fixed{{.*}}!sol.array<5 x ui256, Memory>{{.*}}ui256
// CHECK-DAG:   sol.length {{.*}} : !sol.array<5 x ui256, Memory>

contract C {
    function len_bytes(bytes memory b) public pure returns (uint256) { return b.length; }
    function len_arr(uint256[] memory a) public pure returns (uint256) { return a.length; }
    function len_fixed(uint256[5] memory a) public pure returns (uint256) { return a.length; }
}
