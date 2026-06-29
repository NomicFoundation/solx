// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*a.*}} slot 0 offset 0 : ui8
// CHECK: sol.state_var @{{.*b.*}} slot 0 offset 1 : ui16
// CHECK: sol.state_var @{{.*c.*}} slot 0 offset 3 : i1
// CHECK: sol.state_var @{{.*d.*}} slot 0 offset 4 : !sol.address
// CHECK: sol.state_var @{{.*e.*}} slot 0 offset 24 : !sol.fixedbytes<4>

// CHECK: sol.func @{{.*getA.*}}() -> ui8
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui8, Storage>, ui8

// CHECK: sol.func @{{.*getD.*}}() -> !sol.address
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.address, Storage>, !sol.address

// CHECK: sol.func @{{.*setB.*}}(%arg0: ui16)
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui16, !sol.ptr<ui16, Storage>

// CHECK: sol.func @{{.*setC.*}}(%arg0: i1)
// CHECK:   sol.store %{{.*}}, %{{.*}} : i1, !sol.ptr<i1, Storage>

// CHECK: sol.func @{{.*setE.*}}(%arg0: !sol.fixedbytes<4>)
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, !sol.ptr<!sol.fixedbytes<4>, Storage>

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
