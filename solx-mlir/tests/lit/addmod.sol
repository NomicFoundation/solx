// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*variables.*}}
// CHECK:   sol.addmod {{.*}} : ui256

// CHECK: sol.func @{{.*literals.*}}
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.addmod {{.*}} : ui256

contract C {
    function variables(uint256 x, uint256 y, uint256 m) public pure returns (uint256) {
        return addmod(x, y, m);
    }

    function literals() public pure returns (uint256) {
        return addmod(2, 3, 5);
    }
}
