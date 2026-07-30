// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*}} slot 0 offset 0 : !sol.enum<2>
// CHECK: sol.state_var @{{.*}} slot 1 offset 0 : !sol.mapping<!sol.enum<2>, ui256>
// CHECK: sol.state_var @{{.*}} slot 2 offset 0 : !sol.struct<(!sol.enum<2>, ui256), Storage>

// CHECK: sol.func @{{.*write.*}}
// CHECK:   sol.addr_of @{{.*}} : !sol.ptr<!sol.enum<2>, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.enum<2>, !sol.ptr<!sol.enum<2>, Storage>

// CHECK: sol.func @{{.*read.*}}-> !sol.enum<2>
// CHECK:   sol.addr_of @{{.*}} : !sol.ptr<!sol.enum<2>, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.enum<2>, Storage>, !sol.enum<2>

// CHECK: sol.func @{{.*entry.*}}-> ui256
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.enum<2>, ui256>, !sol.enum<2>, !sol.ptr<ui256, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256

// CHECK: sol.func @{{.*write_entry.*}}
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.enum<2>, ui256>, !sol.enum<2>, !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*field.*}}-> !sol.enum<2>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(!sol.enum<2>, ui256), Storage>, ui64, !sol.ptr<!sol.enum<2>, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.enum<2>, Storage>, !sol.enum<2>

// CHECK: sol.func @{{.*clear.*}}
// CHECK:   sol.constant 0 : ui256
// CHECK:   sol.enum_cast %{{.*}} : ui256 to !sol.enum<2>
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.enum<2>, !sol.ptr<!sol.enum<2>, Storage>

contract C {
    enum E { First, Second, Third }

    struct S { E tag; uint256 amount; }

    E state;
    mapping(E => uint256) counts;
    S item;

    function write(E value) public {
        state = value;
    }

    function read() public view returns (E) {
        return state;
    }

    function entry(E key) public view returns (uint256) {
        return counts[key];
    }

    function write_entry(E key, uint256 value) public {
        counts[key] = value;
    }

    function field() public view returns (E) {
        return item.tag;
    }

    function clear() public {
        delete state;
    }
}
