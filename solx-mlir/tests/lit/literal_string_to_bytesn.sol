// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A string literal used where `bytesN` is expected materialises as a compile-time
// fixed-bytes constant: the literal bytes are left-aligned (high bytes), zero
// padded on the right, into a ui256 constant, then `sol.bytes_cast` to the
// fixedbytes type. Both backends produce the identical big-integer constant and
// bytes_cast op. (solc additionally emits an unused `sol.string_lit`; we pin only
// the shared constant + cast, which is the arm under test.)

// CHECK: sol.func @{{.*f.*}}() -> !sol.fixedbytes<32>
// CHECK:   %[[C:.*]] = sol.constant 47219736118171679016481614208494153725245902603978864281390662590579859259392 : ui256
// CHECK:   %[[B:.*]] = sol.bytes_cast %[[C]] : ui256 to !sol.fixedbytes<32>

contract C {
    function f() public pure returns (bytes32) {
        bytes32 x = "hello";
        return x;
    }
}
