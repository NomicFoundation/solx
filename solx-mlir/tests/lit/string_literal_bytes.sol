// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A string / bytes literal need not be valid UTF-8 (`hex"..."`, escaped bytes);
// it lowers to `sol.string_lit` carrying the raw bytes verbatim.

// CHECK-LABEL: sol.func @{{.*}}raw
// CHECK: sol.string_lit

pragma solidity ^0.8.0;

contract C {
    function raw() external pure returns (bytes memory) {
        return hex"030102ff";
    }
}
