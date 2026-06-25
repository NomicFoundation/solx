// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A derived contract uses an inherited internal helper and an inherited internal
// state variable without overriding either. Both backends copy the inherited
// `helper` body into the concrete @Derived contract and reach it via a direct
// `sol.call`, and access the inherited `stored` slot via addr_of/load. Function
// order is the same on both sides (compute, then helper). Symbol names carry the
// solc node-id suffix (regex). The two backends evaluate the call vs. the storage
// load in opposite order, so those two ops are matched with CHECK-DAG before the
// combining cadd.

// CHECK: sol.contract @{{.*Derived.*}}
// CHECK:   sol.state_var @{{.*stored.*}} slot 0 offset 0 : ui256
// CHECK:   sol.func @{{.*compute.*}}(%{{.*}}: ui256) -> ui256
// CHECK-DAG:     sol.call @{{.*helper.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK-DAG:     sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:     sol.cadd
// CHECK:     sol.return
// CHECK:   sol.func @{{.*helper.*}}(%{{.*}}: ui256) -> ui256
// CHECK:     sol.cadd
// CHECK:     sol.return

contract Base {
    uint256 internal stored;

    function helper(uint256 a) internal pure returns (uint256) {
        return a + a;
    }
}

contract Derived is Base {
    function compute(uint256 x) public view returns (uint256) {
        return helper(x) + stored;
    }
}
