// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `immutable` state variables emit `sol.immutable` (a symbol, NOT a storage slot), are written in
// the constructor through a `!sol.ptr<T, Immutable>` store, and read via `sol.load_immutable`. solx
// and solc agree op-for-op; only symbol node-ids and SSA value names differ (matched by regex).

// CHECK: sol.immutable @{{.*a.*}} : ui256
// CHECK: sol.immutable @{{.*owner.*}} : !sol.address

// CHECK: sol.func {{.*}}kind = #Constructor
// CHECK: sol.addr_of @{{.*a.*}} : !sol.ptr<ui256, Immutable>
// CHECK: sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Immutable>
// CHECK: sol.addr_of @{{.*owner.*}} : !sol.ptr<!sol.address, Immutable>
// CHECK: sol.store %{{.*}}, %{{.*}} : !sol.address, !sol.ptr<!sol.address, Immutable>

// CHECK: sol.func @{{.*getA.*}}
// CHECK: sol.load_immutable @{{.*a.*}} : ui256
// CHECK: sol.func @{{.*getOwner.*}}
// CHECK: sol.load_immutable @{{.*owner.*}} : !sol.address

contract C {
    uint256 immutable a;
    address immutable owner;

    constructor(uint256 v) {
        a = v;
        owner = msg.sender;
    }

    function getA() public view returns (uint256) {
        return a;
    }

    function getOwner() public view returns (address) {
        return owner;
    }
}
