// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `new bytes(n)` / `new string(n)` / `new T[](n)` allocate a dynamically-sized
// memory aggregate of `n` elements via a zero-initialised `sol.malloc`. The
// three functions emit in different orders (solx alphabetical, solc source), so
// match each distinct allocation with CHECK-DAG.

// CHECK-DAG: sol.malloc %{{.*}} zero_init : ui256 !sol.array<? x ui256, Memory>
// CHECK-DAG: sol.malloc %{{.*}} zero_init : ui256 !sol.string<Memory>

contract C {
    function nb(uint256 n) public pure returns (bytes memory) { return new bytes(n); }
    function na(uint256 n) public pure returns (uint256[] memory) { return new uint256[](n); }
    function ns(uint256 n) public pure returns (string memory) { return new string(n); }
}
