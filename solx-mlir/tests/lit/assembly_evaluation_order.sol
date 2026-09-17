// RUN: solx --emit-mlir=sol %s | FileCheck %s

// No print-init RUN line: Yul evaluates an argument list right to left and the C++ frontend
// emits it left to right.

// CHECK: sol.func @{{.*two_calls.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func_call @{{.*right.*}}(
// CHECK:     yul.func_call @{{.*left.*}}(
// CHECK:     yul.add

// CHECK: sol.func @{{.*nested_calls.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func_call @{{.*third.*}}(
// CHECK:     yul.func_call @{{.*second.*}}(
// CHECK:     yul.func_call @{{.*first.*}}(
// CHECK:     yul.func_call @{{.*sum.*}}(

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
