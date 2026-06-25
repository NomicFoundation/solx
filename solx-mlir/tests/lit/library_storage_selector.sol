// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A library external function with a `storage` reference parameter keeps the data-location in its
// canonical selector signature (`g(uint256[] storage)` -> 0xc6bfd994 = -960505452 as i32), matching
// solc; a `memory` parameter does not (`h(uint256[])`). Slang's compute_canonical_signature drops the
// location, so the selector is recomputed from a location-aware signature (see library_aware_selector).

library L {
    function g(uint256[] storage s) external view returns (uint256) { return s.length; }
    function h(uint256[] memory m) external pure returns (uint256) { return m.length; }
}

// CHECK-DAG: sol.func @{{.*}}(%{{.*}}: !sol.array<? x ui256, Storage>) -> ui256 {{.*}}selector = -960505452
// CHECK-DAG: sol.func @{{.*}}(%{{.*}}: !sol.array<? x ui256, Memory>) -> ui256 {{.*}}selector = 1088207624
