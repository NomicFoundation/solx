// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}(%{{.*}}: !sol.struct<(ui256), Memory>) -> ui256 {{.*}}selector = -1327751287
// CHECK: sol.func @{{.*}}(%{{.*}}: !sol.struct<(ui256, ui256), Memory>) -> ui256 {{.*}}selector = -968416976
// CHECK: sol.func @{{.*}}(%{{.*}}: !sol.struct<(ui256, ui256), Storage>) -> ui256 {{.*}}selector = -1043495828
// CHECK: sol.func @{{.*}}(%{{.*}}: !sol.struct<(ui256, ui256), Memory>) -> ui256 {{.*}}selector = 1109318265

pragma abicoder v2;

interface I { struct S { uint256 a; } }

library L {
    struct S { uint256 b; uint256 a; }

    function a(I.S memory) external pure returns (uint256) { return 1; }

    function a(S memory) external pure returns (uint256) { return 2; }

    function f(S storage s) external view returns (uint256) { return s.a; }

    function g(S memory m) external pure returns (uint256) { return m.a; }
}
