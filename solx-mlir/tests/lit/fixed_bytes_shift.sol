// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*shl.*}}
// CHECK: sol.shl %{{[0-9]+}}, %{{[0-9]+}} : !sol.fixedbytes<4>, ui8
// CHECK: sol.func @{{.*shr.*}}
// CHECK: sol.shr %{{[0-9]+}}, %{{[0-9]+}} : !sol.fixedbytes<4>, ui8

contract C {
    function shl(bytes4 a, uint8 n) public pure returns (bytes4) { return a << n; }
    function shr(bytes4 a, uint8 n) public pure returns (bytes4) { return a >> n; }
}
