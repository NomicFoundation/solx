// RUN: solx --emit-mlir %s | FileCheck %s

// Comprehensive test - checks the full Sol dialect module structure.

// CHECK:      module attributes {llvm.data_layout = "E-p:256:256-i256:256:256-S256-a:256:256", llvm.target_triple = "evm-unknown-unknown"
// CHECK:        sol.contract @C {
// CHECK-NEXT:     sol.func @"constructor()"() attributes {kind = #Constructor
// CHECK:            sol.return
// CHECK:          sol.func @"f()"() -> ui256
// CHECK:            %c42_ui8 = sol.constant 42 : ui8
// CHECK-NEXT:       %0 = sol.cast %c42_ui8 : ui8 to ui256
// CHECK-NEXT:       sol.return %0 : ui256
// CHECK:        } {kind = #Contract}

contract C {
    function f() public pure returns (uint256) {
        return 42;
    }
}
