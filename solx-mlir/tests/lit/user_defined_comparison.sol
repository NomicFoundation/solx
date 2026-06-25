// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A user-defined comparison operator (`using {f as ==} for T global;`) dispatches the
// comparison to the bound function instead of emitting a native `sol.cmp`, exactly like
// user-defined arithmetic operators. Both backends call the bound `eq`/`lt`; the symbol
// spelling differs (benign), so the callee is matched loosely.

type Int is int256;
using {eq as ==, lt as <} for Int global;

function eq(Int a, Int b) pure returns (bool) { return Int.unwrap(a) == Int.unwrap(b); }
function lt(Int a, Int b) pure returns (bool) { return Int.unwrap(a) < Int.unwrap(b); }

contract C {
    function useEq(Int a, Int b) public pure returns (bool) { return a == b; }
    function useLt(Int a, Int b) public pure returns (bool) { return a < b; }
}

// CHECK: sol.func @{{.*useEq.*}}
// CHECK: sol.call @{{.*eq.*}}(%{{.*}}, %{{.*}}) : (si256, si256) -> i1
// CHECK: sol.func @{{.*useLt.*}}
// CHECK: sol.call @{{.*lt.*}}(%{{.*}}, %{{.*}}) : (si256, si256) -> i1
