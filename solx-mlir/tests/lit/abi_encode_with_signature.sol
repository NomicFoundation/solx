// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `abi.encodeWithSignature(sig, args)` with a runtime signature hashes `sig`
// with `keccak256`, truncates the digest to its leading four bytes via
// `sol.bytes_cast`, and `sol.encode`s that selector ahead of the arguments.
// (A literal signature is folded to a `ui32` constant instead.)

// CHECK: sol.keccak256{{.*}}!sol.fixedbytes<32>
// CHECK: sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to !sol.fixedbytes<4>
// CHECK: sol.encode selector(%{{[0-9]+}}) {{.*}}!sol.fixedbytes<4> ui256 : !sol.string<Memory>

contract C {
    function f(string memory s) public returns (bytes memory) {
        return abi.encodeWithSignature(s, uint256(1));
    }
}
