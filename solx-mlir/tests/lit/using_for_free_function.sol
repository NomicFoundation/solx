// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// x.dbl() method-call via using-for a free function: solx forwards the receiver
// @dbl(%x) : (ui256); solc's print-init drops it @dbl() : ().

// CHECK: sol.contract @C
// CHECK-SOLX: sol.func @{{.*}}f{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   sol.call @{{.*}}dbl{{.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK-SOLX: sol.func @{{.*}}dbl{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   sol.cmul
// CHECK-SOLC: sol.func @{{.*}}dbl{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   sol.cmul
// CHECK-SOLC: sol.func @{{.*}}f{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   sol.call @{{.*}}dbl{{.*}}() : () -> ui256

function dbl(uint256 a) pure returns (uint256) {
    return a * 2;
}

contract C {
    using {dbl} for uint256;

    function f(uint256 x) public pure returns (uint256) {
        return x.dbl();
    }
}
