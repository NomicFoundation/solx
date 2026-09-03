// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Yul evaluates an argument list right to left - the order the EVM pushes the operands in -
// so a call whose arguments have side effects runs the rightmost one first.

// CHECK: sol.func @{{.*two_calls.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func_call @{{.*right.*}}(
// CHECK:     yul.func_call @{{.*left.*}}(
// CHECK:     yul.add

// Nested calls: the outer argument list is evaluated right to left, and each argument
// evaluates its own list the same way.
// CHECK: sol.func @{{.*nested_calls.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func_call @{{.*third.*}}(
// CHECK:     yul.func_call @{{.*second.*}}(
// CHECK:     yul.func_call @{{.*first.*}}(
// CHECK:     yul.func_call @{{.*sum.*}}(

// A builtin's operands are no different: the literal is materialized before the load. Only
// the emission order reverses - the operand list stays in source order, so `mstore` takes the
// address first.
// CHECK: sol.func @{{.*builtin_operands.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[C:.*]] = yul.constant 32
// CHECK:     %[[XPTR:.*]] = sol.yul_ptr_cast
// CHECK:     %[[X:.*]] = yul.load %[[XPTR]]
// CHECK:     yul.mstore %[[X]], %[[C]]

contract C {
    function two_calls() public returns (uint256 r) {
        assembly {
            function left() -> ret { log0(0, 1) ret := 0x100 }
            function right() -> ret { log0(0, 2) ret := 0x200 }
            r := add(left(), right())
        }
    }

    function nested_calls() public returns (uint256 r) {
        assembly {
            function first() -> ret { log0(0, 1) ret := 0x100 }
            function second() -> ret { log0(0, 2) ret := 0x200 }
            function third() -> ret { log0(0, 3) ret := 0x300 }
            function sum(a, b) -> ret { ret := add(a, b) }
            r := add(sum(first(), second()), third())
        }
    }

    function builtin_operands(uint256 x) public pure {
        assembly {
            mstore(x, 32)
        }
    }
}
