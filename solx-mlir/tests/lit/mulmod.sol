// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*literals.*}}
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.mulmod {{.*}} : ui256

// CHECK: sol.func @{{.*variables.*}}
// CHECK:   sol.mulmod {{.*}} : ui256

contract C {
    function variables(uint256 x, uint256 y, uint256 m) public pure returns (uint256) {
        return mulmod(x, y, m);
    }

    function literals() public pure returns (uint256) {
        return mulmod(2, 3, 5);
    }
}
