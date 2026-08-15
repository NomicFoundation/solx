// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Public-constant auto-getter whose initializer reads a library constant member:
// solc's print-init crashes (SIGSEGV), so this is solx-only.

// CHECK: sol.contract @{{.*C.*}} {
// CHECK: sol.func @{{.*DERIVED.*}}() -> ui256 attributes {orig_fn_type = () -> ui256, selector = 1646776813 : i32, state_mutability = #Pure}
// CHECK:   %[[CONSTANT:.*]] = sol.constant 7 : ui8
// CHECK:   %[[VALUE:.*]] = sol.cast %[[CONSTANT]] : ui8 to ui256
// CHECK:   sol.return %[[VALUE]] : ui256
// CHECK: } {kind = #Contract}

contract C {
    uint256 public constant DERIVED = Library.BASE;
}

library Library {
    uint256 internal constant BASE = 7;
}
