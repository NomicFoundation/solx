// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*ra.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 30 : ui8
// CHECK:   %{{.*}} = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.return %{{.*}} : ui256

// CHECK: sol.func @{{.*rb.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 25 : ui8
// CHECK:   sol.return %{{.*}} : ui256

// CHECK: sol.func @{{.*rc.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 16 : ui8
// CHECK:   sol.return %{{.*}} : ui256

// CHECK: sol.func @{{.*rd.*}}() -> ui8
// CHECK:   %{{.*}} = sol.constant 255 : ui8
// CHECK:   sol.return %{{.*}} : ui8

// CHECK: sol.func @{{.*re.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 1267650600228229401496703205376 : ui104
// CHECK:   %{{.*}} = sol.cast %{{.*}} : ui104 to ui256
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    uint256 constant A = (10 + 5) * 2;
    uint256 constant B = 100 / 4;
    uint256 constant C2 = 1 << 4;
    uint8 constant D = 200 + 55;
    uint256 constant E = 1 << 100;

    function ra() public pure returns (uint256) { return A; }

    function rb() public pure returns (uint256) { return B; }

    function rc() public pure returns (uint256) { return C2; }

    function rd() public pure returns (uint8) { return D; }

    function re() public pure returns (uint256) { return E; }
}
