// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.immutable @{{.*cap.*}} : ui256
// CHECK: sol.immutable @{{.*flag.*}} : i1

// CHECK: sol.func @{{.*check.*}}(%arg0: ui256) -> ui256
// CHECK: %[[X:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: %[[CAP:.*]] = sol.load_immutable @{{.*cap.*}} : ui256
// CHECK: %[[C:.*]] = sol.cmp lt, %[[X]], %[[CAP]] : ui256
// CHECK: sol.require %[[C]]()

contract C {
    uint256 immutable cap;
    bool immutable flag;

    constructor(uint256 c) {
        cap = c;
        flag = c > 0;
    }

    function check(uint256 x) public view returns (uint256) {
        require(x < cap);
        return x;
    }
}
