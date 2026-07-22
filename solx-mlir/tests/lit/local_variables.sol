// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*default_uint.*}}
// CHECK:   sol.constant 0 : ui256

// CHECK: sol.func @{{.*default_address.*}}
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK: sol.func @{{.*default_bytes.*}}
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>

// CHECK: sol.func @{{.*default_bool.*}}
// CHECK:   sol.constant false

// CHECK: sol.func @{{.*explicit_initialize.*}}
// CHECK:   sol.constant 42

// CHECK: sol.func @{{.*reassign.*}}
// CHECK:   sol.cadd
// CHECK:   sol.cmul

contract C {
    function default_uint() public pure returns (uint256) {
        uint256 x;
        return x;
    }

    function default_address() public pure returns (address) {
        address a;
        return a;
    }

    function default_bytes() public pure returns (bytes32) {
        bytes32 b;
        return b;
    }

    function default_bool() public pure returns (bool) {
        bool f;
        return f;
    }

    function explicit_initialize() public pure returns (uint256) {
        uint256 x = 42;
        return x;
    }

    function reassign(uint256 x) public pure returns (uint256) {
        x = x + 1;
        x = x * 2;
        return x;
    }
}
