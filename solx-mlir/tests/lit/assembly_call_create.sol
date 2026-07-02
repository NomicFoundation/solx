// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}
// CHECK: yul.call
// CHECK: yul.static_call
// CHECK: yul.delegate_call
// CHECK: yul.create
// CHECK: yul.create2

contract C {
    function f(address a) public returns (uint256 r) {
        assembly {
            r := call(gas(), a, 0, 0, 0, 0, 0)
            r := staticcall(gas(), a, 0, 0, 0, 0)
            r := delegatecall(gas(), a, 0, 0, 0, 0)
            r := create(0, 0, 0)
            r := create2(0, 0, 0, 0)
        }
    }
}
