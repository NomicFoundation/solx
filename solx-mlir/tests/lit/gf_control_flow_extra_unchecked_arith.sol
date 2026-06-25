// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Inside an `unchecked` block every arithmetic operator lowers to its
// non-checked variant (sol.add/sub/mul/div/mod) on both backends.

// CHECK: sol.func @{{.*f.*}}
// CHECK:   sol.add %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.sub %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.mul %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.div %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.mod %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function f(uint256 a, uint256 b) public pure returns (uint256) {
        unchecked {
            uint256 r = a + b;
            r = r - b;
            r = r * a;
            r = a / b;
            r = a % b;
            return r;
        }
    }
}
