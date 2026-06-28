// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.ext_call {{.*}}callee_type = (!sol.string<CallData>) -> !sol.string<Memory>{{.*}}library_call

library D {
    function f(bytes calldata _x) public pure returns (bytes calldata) { return _x; }
}

contract C {
    using D for bytes;
    function f(bytes calldata _x) public pure returns (bytes1) { return _x.f()[0]; }
}
