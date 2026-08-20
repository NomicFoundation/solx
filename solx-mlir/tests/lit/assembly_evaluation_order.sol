// RUN: solx --emit-mlir=sol %s | FileCheck --check-prefixes=CHECK,CHECK-SOLX %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck --check-prefixes=CHECK,CHECK-SOLC %s

// Yul evaluates an argument list right to left - the order the EVM pushes the operands
// in - so a call whose arguments have side effects runs the rightmost one first. This
// file is where that order is asserted; the C++ frontend still emits left to right, so
// the two are checked separately until it is fixed there too.

// CHECK: sol.func @{{.*two_calls.*}}
// CHECK:   sol.inline_asm {
// CHECK-SOLX:     yul.func_call @{{.*right.*}}(
// CHECK-SOLX:     yul.func_call @{{.*left.*}}(
// CHECK-SOLC:     yul.func_call @{{.*left.*}}(
// CHECK-SOLC:     yul.func_call @{{.*right.*}}(
// CHECK:     yul.add

// Nested calls: the outer argument list is evaluated right to left, and each argument
// evaluates its own list the same way.
// CHECK: sol.func @{{.*nested_calls.*}}
// CHECK:   sol.inline_asm {
// CHECK-SOLX:     yul.func_call @{{.*third.*}}(
// CHECK-SOLX:     yul.func_call @{{.*second.*}}(
// CHECK-SOLX:     yul.func_call @{{.*first.*}}(
// CHECK-SOLX:     yul.func_call @{{.*sum.*}}(
// CHECK-SOLC:     yul.func_call @{{.*first.*}}(
// CHECK-SOLC:     yul.func_call @{{.*second.*}}(
// CHECK-SOLC:     yul.func_call @{{.*sum.*}}(
// CHECK-SOLC:     yul.func_call @{{.*third.*}}(

// A builtin's operands are no different: the literal is materialized before the load.
// Only the emission order reverses - the operand list stays in source order, so `mstore`
// still takes the address first whichever frontend emitted it.
// CHECK: sol.func @{{.*builtin_operands.*}}
// CHECK:   sol.inline_asm {
// CHECK-SOLX:     %[[C:.*]] = yul.constant 32
// CHECK-SOLX:     %[[XPTR:.*]] = sol.yul_ptr_cast
// CHECK-SOLX:     %[[X:.*]] = yul.load %[[XPTR]]
// CHECK-SOLC:     %[[XPTR:.*]] = sol.yul_ptr_cast
// CHECK-SOLC:     %[[X:.*]] = yul.load %[[XPTR]]
// CHECK-SOLC:     %[[C:.*]] = yul.constant 32
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
