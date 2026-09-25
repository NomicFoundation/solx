// RUN: solx --emit-mlir=sol %s | FileCheck %s

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
