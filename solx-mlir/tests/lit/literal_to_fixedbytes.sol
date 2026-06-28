// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A numeric (hex) literal used where `bytesN` is expected materialises as a
// narrow unsigned constant that is `sol.bytes_cast` to the fixed-bytes type
// (with an intermediate widening `sol.cast` when the literal's natural width is
// narrower than N). Functions are alphabetically ordered (b1, b32, b4) so the
// solx (alphabetical) and solc (source-order) walks agree.

// CHECK: sol.func @{{.*}}b1
// CHECK:   sol.constant 18 : ui8
// CHECK:   sol.bytes_cast %{{.*}} : ui8 to !sol.fixedbytes<1>
// CHECK: sol.func @{{.*}}b32
// CHECK:   sol.constant 0 : ui8
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK: sol.func @{{.*}}b4
// CHECK:   sol.constant 2864434397 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

contract C {
    function b1() public pure returns (bytes1) {
        return 0x12;
    }
    function b32() public pure returns (bytes32) {
        return 0x0;
    }
    function b4() public pure returns (bytes4) {
        return 0xaabbccdd;
    }
}
