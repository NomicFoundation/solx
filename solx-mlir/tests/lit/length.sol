// RUN: slang --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func {{.*}}len_arr{{.*}}!sol.array<? x ui256, Memory>{{.*}}ui256
// CHECK:   sol.length {{.*}} : !sol.array<? x ui256, Memory>
// CHECK: sol.func {{.*}}len_bytes{{.*}}!sol.string<Memory>{{.*}}ui256
// CHECK:   sol.length {{.*}} : !sol.string<Memory>
// CHECK: sol.func {{.*}}len_fixed{{.*}}!sol.array<5 x ui256, Memory>{{.*}}ui256
// CHECK:   sol.length {{.*}} : !sol.array<5 x ui256, Memory>
// CHECK: sol.func {{.*}}len_fixed_bytes2{{.*}}!sol.fixedbytes<2>{{.*}}ui256
// CHECK-NOT: sol.length
// CHECK:   sol.constant 2 : ui256
// CHECK: sol.func {{.*}}len_fixed_bytes32{{.*}}!sol.fixedbytes<32>{{.*}}ui256
// CHECK-NOT: sol.length
// CHECK:   sol.constant 32 : ui256
// CHECK: sol.func {{.*}}len_fixed_bytes_xor{{.*}}!sol.fixedbytes<4>{{.*}}ui256
// CHECK-NOT: sol.length
// CHECK:   sol.xor {{.*}} : !sol.fixedbytes<4>
// CHECK-NOT: sol.length
// CHECK:   sol.constant 4 : ui256

contract C {
    function len_bytes(bytes memory b) public pure returns (uint256) { return b.length; }
    function len_arr(uint256[] memory a) public pure returns (uint256) { return a.length; }
    function len_fixed(uint256[5] memory a) public pure returns (uint256) { return a.length; }
    function len_fixed_bytes2(bytes2 b) public pure returns (uint256) { return b.length; }
    function len_fixed_bytes32(bytes32 b) public pure returns (uint256) { return b.length; }
    function len_fixed_bytes_xor(bytes4 a, bytes4 b) public pure returns (uint256) { return (a ^ b).length; }
}
