// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `++` / `--` on a *computed* lvalue (an array element `a[i]` or a struct field
// `s.field`), in both postfix and prefix position. Unlike an identifier lvalue,
// the destination is materialised as an address via `emit_place`, then the
// load / step / store sequence runs against that pointer.
//
// solx and solc emit the four functions in different textual orders, so each
// function's load / step / store / return shape is pinned with CHECK-DAG using
// self-contained captures rather than a single ordered sequence.

// postfix on an array element returns the OLD (loaded) value.
// CHECK-DAG: %[[O1:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui256, Storage>, ui256
// CHECK-DAG: sol.cadd %[[O1]], %{{.*}} : ui256
// CHECK-DAG: sol.return %[[O1]] : ui256

// prefix on an array element returns the NEW (incremented) value.
// CHECK-DAG: %[[N2:.*]] = sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK-DAG: sol.return %[[N2]] : ui256

// prefix decrement on a struct field returns the NEW (decremented) value.
// CHECK-DAG: %[[N4:.*]] = sol.csub %{{.*}}, %{{.*}} : ui256
// CHECK-DAG: sol.return %[[N4]] : ui256

contract C {
    uint256[] arr;
    struct S { uint256 a; }
    S s;

    function postIdx() public returns (uint256) { return arr[0]++; }
    function preIdx() public returns (uint256) { return ++arr[0]; }
    function postField() public returns (uint256) { return s.a++; }
    function preField() public returns (uint256) { return --s.a; }
}
