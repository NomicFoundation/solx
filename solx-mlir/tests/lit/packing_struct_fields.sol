// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A struct whose sub-32-byte fields pack into a single storage slot. Both
// backends lay the struct at slot 0 offset 0 and address fields by GEP on the
// field *index* (a=0, b=1, c=2, d=3), not the packed byte offset. solx emits
// addr_of before the index constant while solc emits the constant first, so the
// addr_of/constant pair is matched with CHECK-DAG. Function source order
// matches the alphabetical walk, so one shared block works.

// CHECK-DAG: sol.state_var @{{.*p.*}} slot 0 offset 0 : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>

// CHECK: sol.func @{{.*readA.*}}() -> ui8
// CHECK-DAG:   sol.addr_of @{{.*p.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>
// CHECK-DAG:   sol.constant 0 : ui64
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>, ui64, !sol.ptr<ui8, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui8, Storage>, ui8

// CHECK: sol.func @{{.*readD.*}}() -> !sol.address
// CHECK-DAG:   sol.addr_of @{{.*p.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>
// CHECK-DAG:   sol.constant 3 : ui64
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>, ui64, !sol.ptr<!sol.address, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.address, Storage>, !sol.address

// CHECK: sol.func @{{.*writeB.*}}(%arg0: ui16)
// CHECK-DAG:   sol.addr_of @{{.*p.*}} : !sol.struct<(ui8, ui16, i1, !sol.address), Storage>
// CHECK-DAG:   sol.constant 1 : ui64
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
