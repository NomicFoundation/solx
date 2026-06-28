// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.state_var @{{.*locked.*}} transient slot 0 offset 0 : i1
// CHECK-DAG: sol.state_var @{{.*owner.*}} transient slot 0 offset 1 : !sol.address

// CHECK: sol.func @{{.*flag.*}}
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*}} : !sol.ptr<i1, Transient>
// CHECK:   sol.load %[[P]] : !sol.ptr<i1, Transient>, i1

// CHECK: sol.func @{{.*lock.*}}
// CHECK-DAG:   sol.constant true
// CHECK-DAG:   sol.addr_of @{{.*}} : !sol.ptr<i1, Transient>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : i1, !sol.ptr<i1, Transient>

// CHECK: sol.func @{{.*setOwner.*}}
// CHECK-DAG:   sol.addr_of @{{.*}} : !sol.ptr<!sol.address, Transient>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : !sol.address, !sol.ptr<!sol.address, Transient>

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
