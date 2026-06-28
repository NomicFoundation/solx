// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

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
