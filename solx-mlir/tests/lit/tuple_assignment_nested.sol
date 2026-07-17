// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc drops an inner element of a parenthesized or nested tuple assignment target, so these cases
// are checked for solx only. TODO: add solc's RUN line once its MLIR backend stops dropping them.

// CHECK: sol.func @{{.*parenthesized_swap.*}}
// CHECK:   %[[B:.*]] = sol.load
// CHECK:   %[[A:.*]] = sol.load
// CHECK:   sol.store %[[B]], %{{.*}}
// CHECK:   sol.store %[[A]], %{{.*}}

// CHECK: sol.func @{{.*nested.*}}
// CHECK:   sol.cast %c1_ui8
// CHECK:   sol.cast %c2_ui8
// CHECK:   sol.cast %c3_ui8

contract C {
    function parenthesized_swap(uint256 a, uint256 b) public pure returns (uint256, uint256) {
        ((a, b)) = (b, a);
        return (a, b);
    }

    function nested() public pure returns (uint256, uint256, uint256) {
        uint256 a;
        uint256 b;
        uint256 c;
        ((a, b), c) = ((1, 2), 3);
        return (a, b, c);
    }
}
