// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*caller.*}}
// CHECK:   %[[A:.*]] = sol.call @{{.*h.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK:   %{{.*}} = sol.call @{{.*h.*}}(%[[A]], %{{.*}}) : (ui256, ui256) -> ui256

contract C {
    function caller() public pure returns (uint256) {
        return h(h(10), 20);
    }

    function h(uint256 a) internal pure returns (uint256) {
        return a + 1;
    }

    function h(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }
}
