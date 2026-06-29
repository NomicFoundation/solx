// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}
// CHECK:   sol.constant 0 : ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address
// CHECK:   sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*}}>
// CHECK:   sol.malloc zero_init : !sol.array<? x ui256, Memory>
// CHECK:   sol.malloc zero_init : !sol.array<3 x ui256, Memory>

interface I {}

contract C {
    function f() public returns (I t, uint256[] memory a, uint256[3] memory b) {}
}
