// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Enum used as a mapping key (!sol.mapping<!sol.enum<1>, ui256>) and as a struct
// field (!sol.struct<(!sol.enum<1>, ui256), Storage>), plus returning the enum
// loaded from storage. State vars are emitted in declaration order on both
// sides; functions differ in order (alphabetical vs source) and the
// addr_of/constant ordering in getItemStatus swaps, so CHECK-DAG is used.

// CHECK-DAG: sol.state_var @{{.*counts.*}} slot 0 offset 0 : !sol.mapping<!sol.enum<1>, ui256>
// CHECK-DAG: sol.state_var @{{.*item.*}} slot 1 offset 0 : !sol.struct<(!sol.enum<1>, ui256), Storage>

// CHECK-DAG: sol.func @{{.*setCount.*}}(%{{.*}}: !sol.enum<1>, %{{.*}}: ui256)
// CHECK-DAG:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.enum<1>, ui256>, !sol.enum<1>, !sol.ptr<ui256, Storage>

// CHECK-DAG: sol.func @{{.*getCount.*}}(%{{.*}}: !sol.enum<1>) -> ui256
// CHECK-DAG:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.enum<1>, ui256>, !sol.enum<1>, !sol.ptr<ui256, Storage>

// CHECK-DAG: sol.func @{{.*getItemStatus.*}}() -> !sol.enum<1>
// CHECK-DAG:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(!sol.enum<1>, ui256), Storage>, ui64, !sol.ptr<!sol.enum<1>, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<!sol.enum<1>, Storage>, !sol.enum<1>

contract C {
    enum Status { Open, Closed }

    struct Item { Status status; uint256 value; }

    mapping(Status => uint256) counts;
    Item item;

    function setCount(Status s, uint256 v) public {
        counts[s] = v;
    }

    function getCount(Status s) public view returns (uint256) {
        return counts[s];
    }

    function getItemStatus() public view returns (Status) {
        return item.status;
    }
}
