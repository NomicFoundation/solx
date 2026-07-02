// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.require {{.*}} "MyError(uint256)"({{.*}}) {{{.*}}call

// CHECK: sol.require {{.*}} "LibraryError(uint256)"({{.*}}) {{{.*}}call

pragma solidity ^0.8.27;

error MyError(uint256 x);

library Library {
    error LibraryError(uint256 x);
}

contract C {
    function f(uint256 a) external pure {
        require(a > 0, MyError(a));
    }
}

contract D {
    function g(uint256 a) external pure {
        require(a > 0, Library.LibraryError(a));
    }
}
