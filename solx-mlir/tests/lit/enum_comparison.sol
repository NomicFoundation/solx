// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Enum declaration plus equality / ordering comparison. Enum values lower to
// !sol.enum<N> (N = max ordinal) and `sol.cmp eq` / `sol.cmp lt` operate on
// that type directly. solx walks functions alphabetically, solc in source
// order; here isEqual precedes isLess in both, so a single CHECK block fits.

// CHECK-DAG: sol.func @{{.*isEqual.*}}(%{{.*}}: !sol.enum<2>, %{{.*}}: !sol.enum<2>) -> i1
// CHECK-DAG:   sol.cmp eq, %{{.*}}, %{{.*}} : !sol.enum<2>

// CHECK-DAG: sol.func @{{.*isLess.*}}(%{{.*}}: !sol.enum<2>, %{{.*}}: !sol.enum<2>) -> i1
// CHECK-DAG:   sol.cmp lt, %{{.*}}, %{{.*}} : !sol.enum<2>

contract C {
    enum Color { Red, Green, Blue }

    function isEqual(Color a, Color b) public pure returns (bool) {
        return a == b;
    }

    function isLess(Color a, Color b) public pure returns (bool) {
        return a < b;
    }
}
