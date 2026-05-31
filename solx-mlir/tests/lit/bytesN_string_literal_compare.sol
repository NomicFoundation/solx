// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A string literal compared with a bytesN operand materializes as a
// fixedbytes constant (not a memory string), so the comparison is emitted
// directly without feeding a string into an integer-only cast.

// CHECK: sol.func @{{.*}}f
// CHECK-NOT: sol.string
// CHECK: sol.cmp

contract C {
    function f(bytes1 b) public pure returns (bool) {
        return b == "d";
    }
}
