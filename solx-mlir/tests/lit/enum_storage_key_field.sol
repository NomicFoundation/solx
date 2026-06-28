// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*counts.*}} slot 0 offset 0 : !sol.mapping<!sol.enum<1>, ui256>
// CHECK: sol.state_var @{{.*item.*}} slot 1 offset 0 : !sol.struct<(!sol.enum<1>, ui256), Storage>

// CHECK: sol.func @{{.*getCount.*}}(%{{.*}}: !sol.enum<1>) -> ui256
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.enum<1>, ui256>, !sol.enum<1>, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*getItemStatus.*}}() -> !sol.enum<1>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(!sol.enum<1>, ui256), Storage>, ui64, !sol.ptr<!sol.enum<1>, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.enum<1>, Storage>, !sol.enum<1>

// CHECK: sol.func @{{.*setCount.*}}(%{{.*}}: !sol.enum<1>, %{{.*}}: ui256)
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.enum<1>, ui256>, !sol.enum<1>, !sol.ptr<ui256, Storage>

contract C {
    enum Status { Open, Closed }

    struct Item { Status status; uint256 value; }

    mapping(Status => uint256) counts;
    Item item;

    function getCount(Status s) public view returns (uint256) {
        return counts[s];
    }

    function getItemStatus() public view returns (Status) {
        return item.status;
    }

    function setCount(Status s, uint256 v) public {
        counts[s] = v;
    }
}
