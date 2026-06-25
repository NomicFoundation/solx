// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Read-modify-write of packed vars sharing slot 0 (uint8 @0, bool @1, uint16 @2).
// Each compound op loads the packed var, computes, and stores back to the same
// offset. The two backends emit the SAME op set per function but schedule them
// differently: solx interleaves addr_of/load/constant while solc front-loads the
// addr_of pair. The bodies are therefore matched with CHECK-DAG. Function source
// order matches the alphabetical walk (bump, flip, widen).

// CHECK-DAG: sol.state_var @{{.*counter.*}} slot 0 offset 0 : ui8
// CHECK-DAG: sol.state_var @{{.*toggled.*}} slot 0 offset 1 : i1
// CHECK-DAG: sol.state_var @{{.*width.*}} slot 0 offset 2 : ui16

// CHECK: sol.func @{{.*bump.*}}()
// CHECK-DAG:   sol.addr_of @{{.*counter.*}} : !sol.ptr<ui8, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui8, Storage>, ui8
// CHECK-DAG:   sol.constant 1 : ui8
// CHECK-DAG:   sol.cadd %{{.*}}, %{{.*}} : ui8
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui8, !sol.ptr<ui8, Storage>

// CHECK: sol.func @{{.*flip.*}}()
// CHECK-DAG:   sol.addr_of @{{.*toggled.*}} : !sol.ptr<i1, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<i1, Storage>, i1
// CHECK-DAG:   sol.constant false
// CHECK-DAG:   sol.cmp eq, %{{.*}}, %{{.*}} : i1
// CHECK:   sol.store %{{.*}}, %{{.*}} : i1, !sol.ptr<i1, Storage>

// CHECK: sol.func @{{.*widen.*}}()
// CHECK-DAG:   sol.addr_of @{{.*width.*}} : !sol.ptr<ui16, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui16, Storage>, ui16
// CHECK-DAG:   sol.constant 5 : ui8
// CHECK-DAG:   sol.cast %{{.*}} : ui8 to ui16
// CHECK-DAG:   sol.cadd %{{.*}}, %{{.*}} : ui16
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui16, !sol.ptr<ui16, Storage>

contract C {
    uint8 counter;
    bool toggled;
    uint16 width;

    function bump() public { counter += 1; }
    function flip() public { toggled = !toggled; }
    function widen() public { width = width + 5; }
}
