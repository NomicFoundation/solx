// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*bitwise_not.*}}
// CHECK: sol.not %{{.*}} : !sol.fixedbytes<32>
// CHECK: sol.func @{{.*xor.*}}
// CHECK: sol.xor %{{.*}}, %{{.*}} : !sol.fixedbytes<4>

contract C {
    function bitwise_not(bytes32 a) public pure returns (bytes32) {
        return ~a;
    }

    function xor(bytes4 a, bytes4 b) public pure returns (bytes4) {
        return a ^ b;
    }
}
