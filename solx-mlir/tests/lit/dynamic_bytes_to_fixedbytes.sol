// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*dyn_to_b32.*}}(%{{.*}}: !sol.string<Memory>) -> !sol.fixedbytes<32>
// CHECK:   sol.dyn_bytes_to_fixedbytes %{{.*}} : <Memory> to <32>

contract C {
    function dyn_to_b32(bytes memory b) public pure returns (bytes32) { return bytes32(b); }
}
