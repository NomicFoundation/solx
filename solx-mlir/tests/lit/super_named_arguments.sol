// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[A:.*]] = sol.cast %c11_ui8
// CHECK: %[[B:.*]] = sol.cast %c99_ui8
// CHECK: sol.call @{{.*}}(%[[A]], %[[B]])

contract A {
    function f(uint256 a, uint256 b) public virtual pure returns (uint256) {
        return a + b;
    }
}

contract B is A {
    function f(uint256 a, uint256 b) public override pure returns (uint256) {
        return super.f({b: 99, a: 11});
    }
}
