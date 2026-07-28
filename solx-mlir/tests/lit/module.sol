// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK:      module attributes {llvm.data_layout = "E-p:256:256-i256:256:256-S256-a:256:256", llvm.target_triple = "evm-unknown-unknown"
// CHECK:        sol.contract @{{.*C.*}} {
// CHECK-NEXT:     sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:            sol.return
// CHECK:          sol.func @{{.*f.*}}() -> ui256
// CHECK:            %c42_ui8 = sol.constant 42 : ui8
// CHECK-NEXT:       %{{.*}} = sol.cast %c42_ui8 : ui8 to ui256
// CHECK-NEXT:       sol.return %{{.*}} : ui256
// CHECK:        } {kind = #Contract}

// CHECK:      module attributes {llvm.data_layout
// CHECK:        sol.contract @{{.*Impl.*}} {
// CHECK:          sol.func @{{.*h.*}}() -> ui256
// CHECK:        } {kind = #Contract}

// CHECK:      module attributes {llvm.data_layout
// CHECK:        sol.contract @{{.*Second.*}} {
// CHECK:          sol.func @{{.*g.*}}() -> ui256
// CHECK:        } {kind = #Contract}

// CHECK-NOT:  sol.contract

contract C {
    function f() public pure returns (uint256) {
        return 42;
    }
}

interface Iface {
    function h() external pure returns (uint256);
}

contract Impl is Iface {
    function h() external pure returns (uint256) {
        return 1;
    }
}

contract Second {
    function g() public pure returns (uint256) {
        return 7;
    }
}

abstract contract Undeployable {
    function h() public pure virtual returns (uint256);
}
