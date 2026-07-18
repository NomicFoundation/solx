// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*identity.*}}
// CHECK-NOT: address_cast

// CHECK: sol.func @{{.*address_to_u160.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to ui160

// CHECK: sol.func @{{.*address_to_u256.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to ui160
// CHECK:   sol.cast %{{.*}} : ui160 to ui256

// CHECK: sol.func @{{.*address_to_bytes20.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to !sol.fixedbytes<20>

// CHECK: sol.func @{{.*to_address.*}}
// CHECK:   sol.cast %{{.*}} : ui256 to ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK: sol.func @{{.*u160_to_address.*}}
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK: sol.func @{{.*literal_to_address.*}}
// CHECK:   sol.constant 0 : ui8
// CHECK:   sol.cast %{{.*}} : ui8 to ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

contract C {
    function identity(address a) public pure returns (address) {
        return address(a);
    }

    function address_to_u160(address a) public pure returns (uint160) {
        return uint160(a);
    }

    function address_to_u256(address a) public pure returns (uint256) {
        return uint256(uint160(a));
    }

    function address_to_bytes20(address a) public pure returns (bytes20) {
        return bytes20(a);
    }

    function to_address(uint256 x) public pure returns (address) {
        return address(uint160(x));
    }

    function u160_to_address(uint160 u) public pure returns (address) {
        return address(u);
    }

    function literal_to_address() public pure returns (address) {
        return address(0);
    }
}
