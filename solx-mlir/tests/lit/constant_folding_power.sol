// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A `constant` whose initializer mixes exponentiation and addition over untyped
// integer literals is fully folded: `K = 2**8 + 1` collapses to the single
// folded value `257 : ui16` (the narrowest type holding 257), then widened to
// the declared ui256 by a cast. Both backends fold identically.

// CHECK: sol.func @{{.*read.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 257 : ui16
// CHECK:   %{{.*}} = sol.cast %{{.*}} : ui16 to ui256
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    uint256 constant K = 2**8 + 1;

    function read() public pure returns (uint256) {
        return K;
    }
}
