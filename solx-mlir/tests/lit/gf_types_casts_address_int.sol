// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// address <-> integer goes through `sol.address_cast` with the integer side
// pinned to `ui160`. address -> ui160 and ui160 -> address are a single
// `address_cast`. address -> uint256 first `address_cast`s to ui160 then a
// plain integer `sol.cast` widens ui160 -> ui256. Both backends agree; function
// order differs so CHECK-DAG is used.

// CHECK-DAG: sol.func @{{.*addr_to_u160.*}}(%{{.*}}: !sol.address) -> ui160
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.address to ui160

// CHECK-DAG: sol.func @{{.*u160_to_addr.*}}(%{{.*}}: ui160) -> !sol.address
// CHECK-DAG:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK-DAG: sol.func @{{.*addr_to_u256.*}}(%{{.*}}: !sol.address) -> ui256
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.address to ui160
// CHECK-DAG:   sol.cast %{{.*}} : ui160 to ui256

contract C {
    function addr_to_u160(address a) public pure returns (uint160) { return uint160(a); }
    function addr_to_u256(address a) public pure returns (uint256) { return uint256(uint160(a)); }
    function u160_to_addr(uint160 u) public pure returns (address) { return address(u); }
}
