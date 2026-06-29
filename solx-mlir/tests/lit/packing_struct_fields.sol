// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*p.*}} slot 0 offset 0 : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>

// CHECK: sol.func @{{.*readA.*}}() -> ui8
// CHECK:   sol.addr_of @{{.*p.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>, ui64, !sol.ptr<ui8, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui8, Storage>, ui8

// CHECK: sol.func @{{.*readD.*}}() -> !sol.address
// CHECK:   sol.addr_of @{{.*p.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>, ui64, !sol.ptr<!sol.address, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.address, Storage>, !sol.address

// CHECK: sol.func @{{.*writeB.*}}(%arg0: ui16)
// CHECK:   sol.addr_of @{{.*p.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>, ui64, !sol.ptr<ui16, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui16, !sol.ptr<ui16, Storage>

contract C {
    struct Packed {
        uint8 a;
        uint16 b;
        bool c;
        address d;
    }

    Packed p;

    function readA() public view returns (uint8) { return p.a; }

    function readD() public view returns (address) { return p.d; }

    function writeB(uint16 v) public { p.b = v; }
}
