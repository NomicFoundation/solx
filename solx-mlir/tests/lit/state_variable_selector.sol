// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// `this.value.selector` on a public state variable folds to its auto-generated
// getter's 4-byte selector — a compile-time `bytes4` constant. Both backends agree
// on the constant (the getter selector for `value()`), bridged via `sol.bytes_cast`.
// They diverge only on the receiver: solx evaluates the `this.value` receiver for
// its side effects first (a `sol.this`), while solc emits the constant directly, so
// the prefixes are split to allow the extra `sol.this` on the solx side.

// CHECK-SOLX: sol.func @{{.*s.*}}() -> !sol.fixedbytes<4>
// CHECK-SOLX:   sol.this : !sol.contract<"C">
// CHECK-SOLX:   sol.constant 1067774533 : ui32
// CHECK-SOLX:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK-SOLX:   sol.return

// CHECK-SOLC: sol.func @{{.*s.*}}() -> !sol.fixedbytes<4>
// CHECK-SOLC:   sol.constant 1067774533 : ui32
// CHECK-SOLC:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK-SOLC:   sol.return

contract C {
    uint256 public value;
    function s() public view returns (bytes4) {
        return this.value.selector;
    }
}
