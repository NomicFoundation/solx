// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.keccak256{{.*}}!sol.fixedbytes<32>
// CHECK: sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to !sol.fixedbytes<4>
// CHECK: sol.encode selector(%{{[0-9]+}}) {{.*}}!sol.fixedbytes<4> ui256 : !sol.string<Memory>

contract C {
    function f(string memory s) public returns (bytes memory) {
        return abi.encodeWithSignature(s, uint256(1));
    }
}
