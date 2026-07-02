// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*isEqual.*}}(%{{.*}}: !sol.enum<2>, %{{.*}}: !sol.enum<2>) -> i1
// CHECK:   sol.cmp eq, %{{.*}}, %{{.*}} : !sol.enum<2>

// CHECK: sol.func @{{.*isLess.*}}(%{{.*}}: !sol.enum<2>, %{{.*}}: !sol.enum<2>) -> i1
// CHECK:   sol.cmp lt, %{{.*}}, %{{.*}} : !sol.enum<2>

contract C {
    enum Color { Red, Green, Blue }

    function isEqual(Color a, Color b) public pure returns (bool) {
        return a == b;
    }

    function isLess(Color a, Color b) public pure returns (bool) {
        return a < b;
    }
}
