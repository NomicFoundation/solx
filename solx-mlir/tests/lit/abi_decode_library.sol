// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// A library name as an abi.decode target type is valid: a library is its address.
// solx decodes the library handle straight to address (sol.decode -> !sol.address);
// solc decodes to the contract type (sol.decode -> !sol.contract<"L">) and also lowers
// the library's internal function, an extra ui8 constant+cast in a pure helper that returns.

// CHECK-SOLC: sol.func @{{.*g.*}}() -> ui256
// CHECK-SOLC: sol.constant 1 : ui8
// CHECK-SOLC: sol.cast %{{.*}} : ui8 to ui256
// CHECK: sol.store %{{.*}}, %{{.*}} : !sol.string<CallData>, !sol.ptr<!sol.string<CallData>, Stack>
// CHECK: %{{.*}} = sol.load %{{.*}} : !sol.ptr<!sol.string<CallData>, Stack>, !sol.string<CallData>
// CHECK-SOLX: sol.decode %{{.*}} : !sol.string<CallData> -> !sol.address
// CHECK-SOLC: sol.decode %{{.*}} : !sol.string<CallData> -> !sol.contract<{{.*}}>

library L {
    function g() internal pure returns (uint256) {
        return 1;
    }
}

contract C {
    function f(bytes calldata d) external pure {
        abi.decode(d, (L));
    }
}
