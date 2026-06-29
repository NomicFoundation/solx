// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// x.inc() method-call via using-for: solx loads x and forwards the receiver
// sol.call @inc(%x) : (ui256) -> ui256; solc print-init drops it to sol.call @inc() : () -> ui256.

// CHECK: sol.contract @C
// CHECK: sol.func @{{.*}}viaMethod{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   %[[Y:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLX:   sol.call @{{.*}}inc{{.*}}(%[[Y]]) : (ui256) -> ui256
// CHECK-SOLC:   sol.call @{{.*}}inc{{.*}}() : () -> ui256

library L {
    function inc(uint256 a) internal pure returns (uint256) {
        return a + 1;
    }
}

contract C {
    using L for uint256;

    function viaMethod(uint256 x) public pure returns (uint256) {
        return x.inc();
    }

    function viaDirect(uint256 x) public pure returns (uint256) {
        return L.inc(x);
    }
}
