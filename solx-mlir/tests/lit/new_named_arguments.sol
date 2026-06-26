// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Named constructor arguments on `new C({...})` are reordered into the
// constructor's declaration order (`{b: 99, a: 11}` -> `ctor(11, 99)`). solc
// reorders identically; pinned solx-only because solc's emission diverges
// benignly (mangled `"D_NN"` contract name).
// CHECK: %[[A:.*]] = sol.cast %c11_ui8
// CHECK: %[[B:.*]] = sol.cast %c99_ui8
// CHECK: sol.new "D"{{.*}}ctor(%[[A]], %[[B]] :

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
