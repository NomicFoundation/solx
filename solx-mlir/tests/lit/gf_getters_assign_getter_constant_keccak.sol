// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Auto-generated getter for a `public constant` whose initializer is NOT an
// integer-foldable form (here `keccak256(...)`): the getter re-emits the
// initializer expression and returns it. Both backends emit the identical
// string_lit -> keccak256 -> return body (solx names it `H()`, solc `get_H_<id>`).

// CHECK: sol.func @{{.*H.*}}() -> !sol.fixedbytes<32> attributes {{.*}}selector = 1889917025 : i32
// CHECK:   %[[S:.*]] = sol.string_lit "x" -> !sol.string<Memory>
// CHECK:   %[[H:.*]] = "sol.keccak256"(%[[S]]) : (!sol.string<Memory>) -> !sol.fixedbytes<32>
// CHECK:   sol.return %[[H]] : !sol.fixedbytes<32>

contract C {
    bytes32 public constant H = keccak256("x");
}
