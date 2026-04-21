// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"to_address(uint256)"
// CHECK:   sol.cast %{{.*}} : ui256 to ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

contract C {
    function to_address(uint256 x) public pure returns (address) {
        return address(uint160(x));
    }
}
