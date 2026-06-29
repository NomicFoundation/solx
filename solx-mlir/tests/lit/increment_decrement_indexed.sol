// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*postField.*}}
// CHECK:   %[[O:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.cadd %[[O]], %{{.*}} : ui256
// CHECK:   sol.return %[[O]] : ui256

// CHECK: sol.func @{{.*postIdx.*}}
// CHECK:   %[[O:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.cadd %[[O]], %{{.*}} : ui256
// CHECK:   sol.return %[[O]] : ui256

// CHECK: sol.func @{{.*preField.*}}
// CHECK:   %[[N:.*]] = sol.csub %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.return %[[N]] : ui256

// CHECK: sol.func @{{.*preIdx.*}}
// CHECK:   %[[N:.*]] = sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.return %[[N]] : ui256

contract C {
    uint256[] arr;
    struct S { uint256 a; }
    S s;

    function postField() public returns (uint256) { return s.a++; }
    function postIdx() public returns (uint256) { return arr[0]++; }
    function preField() public returns (uint256) { return --s.a; }
    function preIdx() public returns (uint256) { return ++arr[0]; }
}
