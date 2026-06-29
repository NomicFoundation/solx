// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*useEq.*}}
// CHECK: sol.call @{{.*eq.*}}(%{{.*}}, %{{.*}}) : (si256, si256) -> i1
// CHECK: sol.func @{{.*useLt.*}}
// CHECK: sol.call @{{.*lt.*}}(%{{.*}}, %{{.*}}) : (si256, si256) -> i1

type Int is int256;
using {eq as ==, lt as <} for Int global;

function eq(Int a, Int b) pure returns (bool) { return Int.unwrap(a) == Int.unwrap(b); }

function lt(Int a, Int b) pure returns (bool) { return Int.unwrap(a) < Int.unwrap(b); }

contract C {
    function useEq(Int a, Int b) public pure returns (bool) { return a == b; }

    function useLt(Int a, Int b) public pure returns (bool) { return a < b; }
}
