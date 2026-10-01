// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func {{.*}}fixed_array{{.*}}!sol.array<4 x ui256, Memory>

// CHECK: sol.func {{.*}}wide_element
// CHECK:   sol.addr_of @{{.*}} : !sol.array<18446744073709551616 x ui256, Storage>

contract C {
    uint256[2**64] wide;

    function fixed_array(uint256[4] memory a) public pure returns (uint256[4] memory) { return a; }

    function wide_element(uint256 i) public view returns (uint256) { return wide[i]; }
}
