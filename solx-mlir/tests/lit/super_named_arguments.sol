// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Named arguments on a `super` call are reordered into the callee's declaration
// order before lowering (here `{b: 99, a: 11}` becomes the positional `(11, 99)`).
// solc reorders identically; this is pinned solx-only because solc's emission
// diverges in benign, pre-existing ways (mangled `@A.f` symbol name).
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
