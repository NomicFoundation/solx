// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Prefix/postfix ++ and -- in statement position inside an `unchecked` block
// lower to the non-checked sol.add / sol.sub (not cadd/csub). Identical on both
// backends.

// CHECK: sol.func @{{.*f.*}}
// CHECK:   sol.add %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.add %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.sub %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.sub %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function f(uint256 a) public pure returns (uint256) {
        unchecked {
            a++;
            ++a;
            a--;
            --a;
        }
        return a;
    }
}
