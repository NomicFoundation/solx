// RUN: solx --emit-mlir=sol %s | FileCheck %s

// An attached public library function returning `bytes calldata` (`using D for bytes; _x.f()`) decodes
// its result into MEMORY across the delegatecall boundary — calldata cannot cross a call. The ext_call's
// callee_type return is `!sol.string<Memory>` (not `<CallData>`), so indexing the result reads the
// returndata rather than the caller's own calldata. The parameter keeps its `CallData` location (it is
// ABI-encoded into the call). solx-only: solc's MLIR frontend NYIs on an attached-library member call
// over `bytes` (SolidityToMLIR.cpp:1698), so there is no print-init parity reference; the runtime
// behaviour is corpus-validated (libraries/attached_public_library_function_returning_calldata).
// FIX: remove all references to solc, solx-solidity repo, and SolidityToMLIR or its other files
library D {
    function f(bytes calldata _x) public pure returns (bytes calldata) { return _x; }
}
contract C {
    using D for bytes;
    function f(bytes calldata _x) public pure returns (bytes1) { return _x.f()[0]; }
}

// CHECK: sol.ext_call {{.*}}callee_type = (!sol.string<CallData>) -> !sol.string<Memory>{{.*}}library_call
