// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Using-for attached library function returning calldata: solc's print-init hits NYI,
// aborting at SolidityToMLIR.cpp:1698 (UNREACHABLE), so this is solx-only.

// CHECK: sol.ext_call {{.*}}callee_type = (!sol.string<CallData>) -> !sol.string<Memory>{{.*}}library_call

library D {
    function f(bytes calldata _x) public pure returns (bytes calldata) { return _x; }
}

contract C {
    using D for bytes;
    function f(bytes calldata _x) public pure returns (bytes1) { return _x.f()[0]; }
}
