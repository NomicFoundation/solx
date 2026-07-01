// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-LABEL: sol.func @{{.*}}raw
// CHECK: sol.string_lit

pragma solidity ^0.8.0;

contract C {
    function raw() external pure returns (bytes memory) {
        return hex"030102ff";
    }
}
