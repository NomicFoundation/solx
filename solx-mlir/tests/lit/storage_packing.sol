// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*tag.*}} slot 0 offset 0 : !sol.fixedbytes<4>
// CHECK: sol.state_var @{{.*small.*}} slot 0 offset 4 : ui8
// CHECK: sol.state_var @{{.*kind.*}} slot 0 offset 5 : !sol.enum<1>
// CHECK: sol.state_var @{{.*owner.*}} slot 0 offset 6 : !sol.address
// CHECK: sol.state_var @{{.*delta.*}} slot 0 offset 26 : si16

// CHECK: sol.func @{{.*readSmall.*}}() -> ui8
// CHECK:   %[[PTR:.*]] = sol.addr_of @{{.*small.*}} : !sol.ptr<ui8, Storage>
// CHECK:   sol.load %[[PTR]] : !sol.ptr<ui8, Storage>, ui8

// CHECK: sol.func @{{.*writeAll.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, !sol.ptr<!sol.fixedbytes<4>, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui8, !sol.ptr<ui8, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.enum<1>, !sol.ptr<!sol.enum<1>, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.address, !sol.ptr<!sol.address, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : si16, !sol.ptr<si16, Storage>

// CHECK: sol.state_var @{{.*callback.*}} slot 0 offset 0 : !sol.func_ref<(ui256) -> ui256>
// CHECK: sol.state_var @{{.*handler.*}} slot 0 offset 8 : !sol.ext_func_ref<(ui256) -> ui256>

// CHECK: sol.state_var @{{.*pair.*}} slot 0 offset 0 : !sol.struct<(ui128, ui64), Storage>

// CHECK: sol.func @{{.*readHi.*}}() -> ui64
// CHECK:   %[[GEP:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui128, ui64), Storage>, ui64, !sol.ptr<ui64, Storage>
// CHECK:   sol.load %[[GEP]] : !sol.ptr<ui64, Storage>, ui64

// CHECK: sol.func @{{.*writeHi.*}}
// CHECK:   %[[GEP:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui128, ui64), Storage>, ui64, !sol.ptr<ui64, Storage>
// CHECK:   sol.store %{{.*}}, %[[GEP]] : ui64, !sol.ptr<ui64, Storage>

enum Kind {
    First,
    Second
}

contract Kinds {
    bytes4 tag;
    uint8 small;
    Kind kind;
    address owner;
    int16 delta;

    function readSmall() public view returns (uint8) {
        return small;
    }

    function writeAll(bytes4 t, uint8 s, Kind k, address o, int16 d) public {
        tag = t;
        small = s;
        kind = k;
        owner = o;
        delta = d;
    }
}

contract Pointers {
    function (uint256) internal returns (uint256) callback;
    function (uint256) external returns (uint256) handler;
}

contract Structs {
    struct Pair {
        uint128 lo;
        uint64 hi;
    }

    Pair pair;

    function readHi() public view returns (uint64) {
        return pair.hi;
    }

    function writeHi(uint64 value) public {
        pair.hi = value;
    }
}
