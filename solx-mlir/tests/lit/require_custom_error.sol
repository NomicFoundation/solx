// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// require(cond, CustomError(args)) (Solidity >= 0.8.26) lowers to the `call`
// form of sol.require carrying the error's canonical signature and arguments,
// exactly as `revert CustomError(args)` does. solx and solc agree.

// FIX: merge CHECK sequences and comments into their own blocks
// CHECK: sol.require {{.*}} "MyError(uint256)"({{.*}}) {{{.*}}call

// The error operand may also be a member access (an error declared in a library,
// referenced as Lib.LibError(args)); it reaches the same call form.
// CHECK: sol.require {{.*}} "LibError(uint256)"({{.*}}) {{{.*}}call

pragma solidity ^0.8.27;

error MyError(uint256 x);

library Lib {
    error LibError(uint256 x);
}

contract C {
    function f(uint256 a) external pure {
        require(a > 0, MyError(a));
    }
}

contract D {
    function g(uint256 a) external pure {
        require(a > 0, Lib.LibError(a));
    }
}
