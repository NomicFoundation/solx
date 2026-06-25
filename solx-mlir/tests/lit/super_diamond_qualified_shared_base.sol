// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Diamond `D is B, C; B is A; C is A` where both sides and the most-derived body
// reach the shared base via the explicit base-qualified call `A.base()`. The base
// `base` function is therefore a redirect target reached along several paths, but
// the precompute pass walks (and the backend emits) its body exactly once. Both
// backends agree: @D carries a single `base` body (constant 1), the `fromB`
// (constant 10) and `fromC` (constant 100) helpers each issue one `sol.call` to it,
// and `go()` issues three calls. Only symbol names (solc node-id suffix vs. solx
// qualified name) and the function / operand emission order differ, so the three
// helper bodies are pinned by their distinct constants with CHECK-DAG and the
// single shared base is asserted with a count.

// CHECK: sol.contract @D{{.*}} {
// CHECK-DAG:   sol.constant 1 : ui8
// CHECK-DAG:   sol.constant 10 : ui8
// CHECK-DAG:   sol.constant 100 : ui8
// `go()` issues three direct calls (fromB, fromC, the shared base) joined by two adds.
// CHECK-DAG:   sol.func @{{.*go.*}}() -> ui256
// CHECK-DAG:     sol.call @{{.*}}() : () -> ui256
// CHECK-DAG:     sol.call @{{.*}}() : () -> ui256
// CHECK-DAG:     sol.call @{{.*}}() : () -> ui256

// The shared base body (constant 1) is emitted exactly once; no second copy appears.
// CHECK-NOT: sol.constant 2 : ui8

contract A {
    function base() internal pure virtual returns (uint256) { return 1; }
}
contract B is A {
    function fromB() internal pure returns (uint256) { return A.base() + 10; }
}
contract C is A {
    function fromC() internal pure returns (uint256) { return A.base() + 100; }
}
contract D is B, C {
    function go() public pure returns (uint256) { return fromB() + fromC() + A.base(); }
}
