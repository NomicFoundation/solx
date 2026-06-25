// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Ternary over non-uint scalar value types: an address result slot and a signed
// int256 result slot. solx walks functions alphabetically (taddr, tsigned), solc
// in source order (taddr, tsigned) here too, but the slot element type makes each
// branch store unambiguous, so CHECK-DAG pins each independently of order.

// CHECK-DAG: sol.func @{{.*taddr.*}}(%{{.*}}: i1, %{{.*}}: !sol.address, %{{.*}}: !sol.address) -> !sol.address
// CHECK-DAG:   sol.alloca : !sol.ptr<!sol.address, Stack>

// CHECK-DAG: sol.func @{{.*tsigned.*}}(%{{.*}}: i1, %{{.*}}: si256, %{{.*}}: si256) -> si256
// CHECK-DAG:   sol.alloca : !sol.ptr<si256, Stack>

contract C {
    function taddr(bool c, address a, address b) public pure returns (address) {
        return c ? a : b;
    }
    function tsigned(bool c, int256 a, int256 b) public pure returns (int256) {
        return c ? a : b;
    }
}
