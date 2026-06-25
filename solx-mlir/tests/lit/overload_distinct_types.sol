// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Overloading `k` by two distinct non-integer ABI types (address vs bytes32).
// Each overload lowers to a distinct sol type (!sol.address vs
// !sol.fixedbytes<32>) and a distinct selector (478120042 vs -167583623).
// Both backends produce identical signatures and selectors; only the symbol
// name suffix (_<nodeid> on solc) differs. CHECK-DAG matches each overload
// independently of emission order.

// CHECK-DAG: sol.func @{{.*k.*}}(%{{.*}}: !sol.address) -> !sol.address attributes {{.*}}selector = 478120042 : i32
// CHECK-DAG: sol.func @{{.*k.*}}(%{{.*}}: !sol.fixedbytes<32>) -> !sol.fixedbytes<32> attributes {{.*}}selector = -167583623 : i32

contract C {
    function k(address a) public pure returns (address) {
        return a;
    }

    function k(bytes32 a) public pure returns (bytes32) {
        return a;
    }
}
