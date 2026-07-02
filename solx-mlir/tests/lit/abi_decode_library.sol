// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// abi.decode target is library L: solx decodes the handle to !sol.address and drops L's body.
// solc decodes to !sol.contract<"L"> and still lowers L.g (ui8 constant 1 cast to ui256).

// CHECK-SOLC: sol.func @{{.*g.*}}() -> ui256
// CHECK-SOLC: sol.constant 1 : ui8
// CHECK-SOLC: sol.cast %{{.*}} : ui8 to ui256
// CHECK: sol.store %arg0, %{{.*}} : !sol.string<CallData>, !sol.ptr<!sol.string<CallData>, Stack>
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
