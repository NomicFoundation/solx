// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}(%{{.*}}: !sol.array<? x ui256, Storage>) -> ui256 {{.*}}selector = -960505452
// CHECK: sol.func @{{.*}}(%{{.*}}: !sol.array<? x ui256, Memory>) -> ui256 {{.*}}selector = 1088207624

library L {
    function g(uint256[] storage s) external view returns (uint256) { return s.length; }

    function h(uint256[] memory m) external pure returns (uint256) { return m.length; }
}
