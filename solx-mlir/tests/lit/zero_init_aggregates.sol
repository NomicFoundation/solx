// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Default zero-initialization of non-integer named returns. A contract/interface
// reference zeroes as `address(0)` reinterpreted to the contract type (two
// `sol.address_cast`s). A memory array — dynamic or fixed — zeroes to a fresh
// `sol.malloc zero_init` buffer. (A storage array reference and a mapping have
// no such default and are handled elsewhere / left to the caller.)

// CHECK: sol.func @{{.*f.*}}
// CHECK: sol.constant 0 : ui160
// CHECK: sol.address_cast %{{.*}} : ui160 to !sol.address
// CHECK: sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*}}>
// CHECK-DAG: sol.malloc zero_init : !sol.array<? x ui256, Memory>
// CHECK-DAG: sol.malloc zero_init : !sol.array<3 x ui256, Memory>

interface I {}

contract C {
    function f() public returns (I t, uint256[] memory a, uint256[3] memory b) {}
}
