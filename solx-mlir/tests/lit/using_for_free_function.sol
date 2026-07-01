// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// x.double() method-call via using-for a free function: solx forwards the receiver
// @double(%x) : (ui256); solc's print-init drops it @double() : ().

// CHECK: sol.contract @C
// CHECK-SOLX: sol.func @{{.*}}f{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   sol.call @{{.*}}double{{.*}}_[[D:[0-9]+]]"(%{{.*}}) : (ui256) -> ui256
// CHECK-SOLX: sol.func @{{.*}}double{{.*}}_[[D]]"(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   sol.cmul
// CHECK-SOLC: sol.func @{{.*}}double{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   sol.cmul
// CHECK-SOLC: sol.func @{{.*}}f{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   sol.call @{{.*}}double{{.*}}() : () -> ui256

function double(uint256 a) pure returns (uint256) {
    return a * 2;
}

contract C {
    using {double} for uint256;

    function f(uint256 x) public pure returns (uint256) {
        return x.double();
    }
}
