// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Deeply nested plain `{ }` scoped blocks: both backends flatten them into a
// single stack frame (four allocas) and chain the additions; the final scoped
// `a = a + b + c + d` becomes three sol.cadd ops.

// CHECK: sol.func @{{.*deep.*}}
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function deep() public pure returns (uint256) {
        uint256 a = 1;
        {
            uint256 b = 2;
            {
                uint256 c = 3;
                {
                    uint256 d = 4;
                    a = a + b + c + d;
                }
            }
        }
        return a;
    }
}
