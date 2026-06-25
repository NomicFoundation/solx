// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Two functions named `f` overloaded by parameter count. Both backends mangle
// the public selector identically (-1277270901 for f(uint256), 332507694 for
// f(uint256,uint256)), so the differing symbol names (solc appends _<nodeid>)
// are matched with a regex and the selectors are pinned exactly. Functions are
// matched with CHECK-DAG since solx orders alphabetically and solc in source
// order; here that order coincides, but CHECK-DAG keeps the test robust.

// CHECK-DAG: sol.func @{{.*f.*}}(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector = -1277270901 : i32
// CHECK-DAG:   sol.cadd %{{.*}}, %{{.*}} : ui256

// CHECK-DAG: sol.func @{{.*f.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256 attributes {{.*}}selector = 332507694 : i32
// CHECK-DAG:   sol.cadd %{{.*}}, %{{.*}} : ui256

contract C {
    function f(uint256 a) public pure returns (uint256) {
        return a + 1;
    }

    function f(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}
