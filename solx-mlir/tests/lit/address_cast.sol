// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*to_address.*}}
// CHECK:   sol.cast %{{.*}} : ui256 to ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

contract C {
    function to_address(uint256 x) public pure returns (address) {
        return address(uint160(x));
    }
}
