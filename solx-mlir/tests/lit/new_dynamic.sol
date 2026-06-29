// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*na.*}}
// CHECK:   sol.malloc %{{.*}} zero_init : ui256 !sol.array<? x ui256, Memory>
// CHECK: sol.func @{{.*nb.*}}
// CHECK:   sol.malloc %{{.*}} zero_init : ui256 !sol.string<Memory>
// CHECK: sol.func @{{.*newString.*}}
// CHECK:   sol.malloc %{{.*}} zero_init : ui256 !sol.string<Memory>

contract C {
    function na(uint256 n) public pure returns (uint256[] memory) { return new uint256[](n); }

    function nb(uint256 n) public pure returns (bytes memory) { return new bytes(n); }

    function newString(uint256 n) public pure returns (string memory) { return new string(n); }
}
