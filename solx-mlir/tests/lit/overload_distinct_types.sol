// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*k.*}}(%{{.*}}: !sol.address) -> !sol.address attributes {{.*}}selector = 478120042 : i32
// CHECK: sol.func @{{.*k.*}}(%{{.*}}: !sol.fixedbytes<32>) -> !sol.fixedbytes<32> attributes {{.*}}selector = -167583623 : i32

contract C {
    function k(address a) public pure returns (address) {
        return a;
    }

    function k(bytes32 a) public pure returns (bytes32) {
        return a;
    }
}
