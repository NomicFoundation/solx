// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*locals.*}}
// CHECK: sol.conv_cast %{{.*}} : !sol.ptr<ui256, Stack> -> !llvm.ptr
// CHECK: llvm.load %{{.*}} : !llvm.ptr -> i256
// CHECK: llvm.alloca %{{.*}} x i256
// CHECK: llvm.store %{{.*}} : i256, !llvm.ptr

contract C {
    function locals(uint256 a) public pure returns (uint256 r) {
        assembly {
            let x := a
            let y := mul(x, 2)
            r := add(x, y)
        }
    }
}
