// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*addrDefault.*}}() -> !sol.address
// CHECK:   %[[A:.*]] = sol.constant 0 : ui160
// CHECK:   %{{.*}} = sol.address_cast %[[A]] : ui160 to !sol.address

// CHECK: sol.func @{{.*boolDefault.*}}() -> i1
// CHECK:   %{{.*}} = sol.constant false

// CHECK: sol.func @{{.*bytes4Default.*}}() -> !sol.fixedbytes<4>
// CHECK:   %[[BZ:.*]] = sol.constant 0 : ui32
// CHECK:   %{{.*}} = sol.bytes_cast %[[BZ]] : ui32 to !sol.fixedbytes<4>

// CHECK: sol.func @{{.*bytesDefault.*}}() -> !sol.string<Memory>

// CHECK: sol.func @{{.*dynArrDefault.*}}() -> !sol.array<? x ui256, Memory>
// CHECK:   %{{.*}} = sol.malloc zero_init :  !sol.array<? x ui256, Memory>

// CHECK: sol.func @{{.*enumDefault.*}}() -> !sol.enum<1>
// CHECK:   %[[EZ:.*]] = sol.constant 0 : ui256
// CHECK:   %{{.*}} = sol.enum_cast %[[EZ]] : ui256 to !sol.enum<1>

// CHECK: sol.func @{{.*fixedArrDefault.*}}() -> !sol.array<2 x ui256, Memory>
// CHECK:   %{{.*}} = sol.malloc zero_init :  !sol.array<2 x ui256, Memory>

// CHECK: sol.func @{{.*stringDefault.*}}() -> !sol.string<Memory>

// CHECK: sol.func @{{.*structDefault.*}}() -> !sol.struct<(ui256), Memory>
// CHECK:   %{{.*}} = sol.malloc zero_init :  !sol.struct<(ui256), Memory>

// CHECK: sol.func @{{.*uintDefault.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 0 : ui256

contract C {
    enum E { A, B }
    struct S { uint256 x; }

    function addrDefault() public pure returns (address) {}
    function boolDefault() public pure returns (bool) {}
    function bytes4Default() public pure returns (bytes4) {}
    function bytesDefault() public pure returns (bytes memory) {}
    function dynArrDefault() public pure returns (uint256[] memory) {}
    function enumDefault() public pure returns (E) {}
    function fixedArrDefault() public pure returns (uint256[2] memory) {}
    function stringDefault() public pure returns (string memory) {}
    function structDefault() public pure returns (S memory) {}
    function uintDefault() public pure returns (uint256) {}
}
