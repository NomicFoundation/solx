// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[A:.*]] = sol.cast %c11_ui8
// CHECK: %[[B:.*]] = sol.cast %c99_ui8
// CHECK: sol.new "D{{.*}}ctor(%[[A]], %[[B]] :

contract D {
    uint256 x;
    uint256 y;

    constructor(uint256 a, uint256 b) {
        x = a;
        y = b;
    }
}

contract C {
    function make() external returns (address) {
        return address(new D({b: 99, a: 11}));
    }
}
