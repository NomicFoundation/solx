// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// address <-> integer routes through sol.address_cast with the integer side pinned
// to ui160; widening past ui160 (address to uint256) appends a plain integer sol.cast.

// CHECK-DAG: sol.func @{{.*to_address.*}}
// CHECK-DAG:   sol.cast %{{.*}} : ui256 to ui160
// CHECK-DAG:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK-DAG: sol.func @{{.*addr_to_u160.*}}
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.address to ui160

// CHECK-DAG: sol.func @{{.*addr_to_u256.*}}
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.address to ui160
// CHECK-DAG:   sol.cast %{{.*}} : ui160 to ui256

// CHECK-DAG: sol.func @{{.*u160_to_addr.*}}
// CHECK-DAG:   sol.address_cast %{{.*}} : ui160 to !sol.address

contract C {
    function to_address(uint256 x) public pure returns (address) {
        return address(uint160(x));
    }

    function addr_to_u160(address a) public pure returns (uint160) {
        return uint160(a);
    }

    function addr_to_u256(address a) public pure returns (uint256) {
        return uint256(uint160(a));
    }

    function u160_to_addr(uint160 u) public pure returns (address) {
        return address(u);
    }
}
