// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A Yul local variable lives in an `llvm.alloca` slot loaded/stored as `i256`.
// A Solidity variable referenced from assembly is reached by reinterpreting its
// `!sol.ptr<…, Stack>` as `!llvm.ptr` via `sol.conv_cast`, then a plain
// `llvm.load`/`llvm.store` — the sole Sol/Yul representation boundary (rule 16).

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
