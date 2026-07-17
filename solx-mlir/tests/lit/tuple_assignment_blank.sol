// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc emits nothing for a file containing a blank tuple target, so this case is checked for solx
// only. TODO: fold into tuple_assignment.sol once solc's MLIR backend compiles blank targets.

// CHECK: sol.func @{{.*blank.*}}
// CHECK: sol.cast %c7_ui8

contract C {
    function blank() public pure returns (uint256) {
        uint256 a;
        (a, ) = (7, 8);
        return a;
    }
}
