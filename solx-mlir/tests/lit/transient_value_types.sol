// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*locked.*}} transient slot 0 offset 0 : i1
// CHECK: sol.state_var @{{.*owner.*}} transient slot 0 offset 1 : !sol.address

// CHECK: sol.func @{{.*flag.*}}
// CHECK:   sol.load %{{.*}} : !sol.ptr<i1, Transient>, i1

// CHECK: sol.func @{{.*lock.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : i1, !sol.ptr<i1, Transient>

// CHECK: sol.func @{{.*setOwner.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.address, !sol.ptr<!sol.address, Transient>

contract C {
    bool transient locked;
    address transient owner;

    function flag() public view returns (bool) {
        return locked;
    }

    function lock() public {
        locked = true;
    }

    function setOwner(address a) public {
        owner = a;
    }
}
