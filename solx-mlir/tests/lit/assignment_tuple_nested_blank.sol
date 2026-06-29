// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Nested tuple LHS with a blank slot: solx lowers both and stores right-to-left.
// solc aborts in genLValExpr (res.size() == 1) on the nested tuple, so it cannot compile this file.

// CHECK: sol.func @"blank()"
// CHECK: %[[B:.*]] = sol.cast %c7_ui8 : ui8 to ui256
// CHECK: sol.store %[[B]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>

// CHECK: sol.func @"nested()"
// CHECK: %[[C:.*]] = sol.cast %c3_ui8 : ui8 to ui256
// CHECK: sol.store %[[C]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>
// CHECK: %[[VB:.*]] = sol.cast %c2_ui8 : ui8 to ui256
// CHECK: sol.store %[[VB]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>
// CHECK: %[[VA:.*]] = sol.cast %c1_ui8 : ui8 to ui256
// CHECK: sol.store %[[VA]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>

contract C {
    function nested() public pure returns (uint256, uint256, uint256) {
        uint256 a; uint256 b; uint256 c;
        ((a, b), c) = ((1, 2), 3);
        return (a, b, c);
    }

    function blank() public pure returns (uint256) {
        uint256 a;
        (a, ) = (7, 8);
        return a;
    }
}
