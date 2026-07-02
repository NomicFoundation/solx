// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*H.*}}() -> !sol.fixedbytes<32> attributes {{.*}}selector = 1889917025 : i32
// CHECK:   %[[S:.*]] = sol.string_lit "x" -> !sol.string<Memory>
// CHECK:   %[[H:.*]] = "sol.keccak256"(%[[S]]) : (!sol.string<Memory>) -> !sol.fixedbytes<32>
// CHECK:   sol.return %[[H]] : !sol.fixedbytes<32>

contract C {
    bytes32 public constant H = keccak256("x");
}
