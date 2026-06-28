// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: %[[O1:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui256, Storage>, ui256
// CHECK-DAG: sol.cadd %[[O1]], %{{.*}} : ui256
// CHECK-DAG: sol.return %[[O1]] : ui256

// CHECK-DAG: %[[N2:.*]] = sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK-DAG: sol.return %[[N2]] : ui256

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
