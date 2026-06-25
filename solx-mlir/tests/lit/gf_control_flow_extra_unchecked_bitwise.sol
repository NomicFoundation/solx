// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Exponentiation, shifts and bitwise operators inside an `unchecked` block.
// Both backends emit the same op set (operand SSA order differs, so use
// regex placeholders for the operands).

// CHECK: sol.func @{{.*f.*}}
// CHECK:   sol.exp %{{.*}}, %{{.*}} : ui256, ui256 -> ui256
// CHECK:   sol.shl %{{.*}}, %{{.*}} : ui256, ui256
// CHECK:   sol.shr %{{.*}}, %{{.*}} : ui256, ui256
// CHECK:   sol.and %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.or %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.xor %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    function f(uint256 a, uint256 b) public pure returns (uint256) {
        unchecked {
            uint256 r = a ** b;
            r = r << b;
            r = r >> a;
            r = a & b;
            r = a | b;
            r = a ^ b;
            return r;
        }
    }
}
