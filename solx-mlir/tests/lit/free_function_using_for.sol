// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init drops the receiver from a using-for free-function call, calling
// `@double() : ()`, while legacy forwards it, so this is solx-only.

// CHECK: sol.func @{{.*}}f{{.*}}(%{{.*}}: ui256) -> ui256
// CHECK:   %[[R:.*]] = sol.call @"double(uint256)_[[D:[0-9]+]]"(%{{.*}}) : (ui256) -> ui256
// CHECK:   sol.return %[[R]] : ui256
// CHECK: sol.func @"double(uint256)_[[D]]"(%{{.*}}: ui256) -> ui256
// CHECK:   sol.cmul

function double(uint256 a) pure returns (uint256) {
    return a * 2;
}

contract C {
    using {double} for uint256;

    function f(uint256 x) public pure returns (uint256) {
        return x.double();
    }
}
