// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Five sub-32-byte state vars pack into slot 0 at distinct byte offsets:
// uint8 @0, uint16 @1, bool @3, address @4, bytes4 @24. Both backends compute
// identical slot/offset assignments. Reads are addr_of -> load; writes are
// addr_of + load(stack) + store, but solx emits load-before-addr_of while solc
// emits addr_of-before-load, so the write ops are pinned with CHECK-DAG.
// Function source order matches the alphabetical walk, so a single block works.

// CHECK-DAG: sol.state_var @{{.*a.*}} slot 0 offset 0 : ui8
// CHECK-DAG: sol.state_var @{{.*b.*}} slot 0 offset 1 : ui16
// CHECK-DAG: sol.state_var @{{.*c.*}} slot 0 offset 3 : i1
// CHECK-DAG: sol.state_var @{{.*d.*}} slot 0 offset 4 : !sol.address
// CHECK-DAG: sol.state_var @{{.*e.*}} slot 0 offset 24 : !sol.fixedbytes<4>

// CHECK: sol.func @{{.*getA.*}}() -> ui8
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*a.*}} : !sol.ptr<ui8, Storage>
// CHECK:   sol.load %[[P]] : !sol.ptr<ui8, Storage>, ui8

// CHECK: sol.func @{{.*getD.*}}() -> !sol.address
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*d.*}} : !sol.ptr<!sol.address, Storage>
// CHECK:   sol.load %[[P]] : !sol.ptr<!sol.address, Storage>, !sol.address

// CHECK: sol.func @{{.*setB.*}}(%arg0: ui16)
// CHECK-DAG:   sol.addr_of @{{.*b.*}} : !sol.ptr<ui16, Storage>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui16, !sol.ptr<ui16, Storage>

// CHECK: sol.func @{{.*setE.*}}(%arg0: !sol.fixedbytes<4>)
// CHECK-DAG:   sol.addr_of @{{.*e.*}} : !sol.ptr<!sol.fixedbytes<4>, Storage>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, !sol.ptr<!sol.fixedbytes<4>, Storage>

contract C {
    uint8 a;
    uint16 b;
    bool c;
    address d;
    bytes4 e;

    function getA() public view returns (uint8) { return a; }
    function getD() public view returns (address) { return d; }
    function setB(uint16 v) public { b = v; }
    function setC(bool v) public { c = v; }
    function setE(bytes4 v) public { e = v; }
}
