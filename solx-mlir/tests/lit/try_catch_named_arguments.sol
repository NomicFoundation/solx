// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: try_call
// CHECK: sol.try
// CHECK: fallback {

contract A {
    function g(uint256 a, uint256 b) external pure returns (uint256) {
        return a - b;
    }
}

contract C {
    function f(A inst) external returns (uint256) {
        try inst.g({b: 11, a: 99}) returns (uint256 r) {
            return r;
        } catch {
            return 0;
        }
    }
}
