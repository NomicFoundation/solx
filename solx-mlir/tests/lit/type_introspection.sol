// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// type(T).max / type(T).min / type(I).interfaceId are compile-time constants;
// solx names the functions by canonical signature, solc by `<name>_<id>`.

// CHECK-SOLX: sol.func @"umax()"() -> ui8
// CHECK-SOLC: sol.func @umax_{{[0-9]+}}() -> ui8
// CHECK:   sol.constant 255 : ui8

// CHECK-SOLX: sol.func @"imin()"() -> si8
// CHECK-SOLC: sol.func @imin_{{[0-9]+}}() -> si8
// CHECK:   sol.constant -128 : si8

// CHECK-SOLX: sol.func @"iid()"() -> !sol.fixedbytes<4>
// CHECK-SOLC: sol.func @iid_{{[0-9]+}}() -> !sol.fixedbytes<4>
// CHECK:   sol.constant 801029432 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

interface I {
    function foo(uint256) external;
}

contract C {
    function umax() public pure returns (uint8) {
        return type(uint8).max;
    }

    function imin() public pure returns (int8) {
        return type(int8).min;
    }

    function iid() public pure returns (bytes4) {
        return type(I).interfaceId;
    }
}
