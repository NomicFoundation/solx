// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A `bytesN` value read in inline assembly is reinterpreted as its raw,
// left-aligned 256-bit stack word via `sol.conv_cast`, not shifted as a
// value conversion.

// CHECK: sol.func @{{.*}}f
// CHECK: sol.conv_cast

contract C {
    function f() public pure returns (uint256 r) {
        bytes2 y = 0xffff;
        assembly {
            r := y
        }
    }
}
