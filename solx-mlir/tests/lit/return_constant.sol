// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Comprehensive test - checks the full Sol dialect module structure.

// CHECK:      module attributes {llvm.data_layout = "E-p:256:256-i256:256:256-S256-a:256:256", llvm.target_triple = "evm-unknown-unknown"
// CHECK:        sol.contract @{{.*C.*}} {
// CHECK-NEXT:     sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:            sol.return
// CHECK:          sol.func @{{.*f.*}}() -> ui256
// CHECK:            %c42_ui8 = sol.constant 42 : ui8
// CHECK-NEXT:       %{{.*}} = sol.cast %c42_ui8 : ui8 to ui256
// CHECK-NEXT:       sol.return %{{.*}} : ui256
// CHECK:        } {kind = #Contract}

contract C {
    function f() public pure returns (uint256) {
        return 42;
    }
}
