// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `T.wrap(x)` / `T.unwrap(x)` route through the call-expression built-in arm and
// convert to the result type; for a UDVT over `uint256` the representation is
// identical, so both lower to a pass-through with the value returned unchanged.
// The symbol names diverge (solc appends a node id), so identify each function
// by its (identical) selector. The two functions emit in different orders (solx
// alphabetical, solc source), so match with CHECK-DAG.

// CHECK-DAG: sol.func @{{.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -1889074879
// CHECK-DAG: sol.func @{{.*}}(%arg0: ui256) -> ui256 attributes {{.*}}selector = -859207792

type Decimal is uint256;

contract C {
    function w(uint256 x) public pure returns (Decimal) { return Decimal.wrap(x); }
    function u(Decimal d) public pure returns (uint256) { return Decimal.unwrap(d); }
}
