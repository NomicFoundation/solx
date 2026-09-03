// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Every Yul literal reaches the dialect as a signless i256 word: a bool is 1 or 0, and a
// string or `hex"..."` literal is left-aligned in the word rather than zero-extended.

// CHECK: sol.func @{{.*numbers.*}}
// CHECK:   sol.inline_asm {
// CHECK-DAG:     yul.constant 42
// CHECK-DAG:     yul.constant 255
// CHECK:     yul.add
// A word of all ones prints signed.
// CHECK:     yul.constant -1
// CHECK:     yul.add

// CHECK: sol.func @{{.*booleans.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[TRUE:.*]] = yul.constant 1
// CHECK:     yul.if %[[TRUE]] {
// CHECK:     yul.cmp eq, %{{.*}}, %{{.*}}
// CHECK:     yul.if

// CHECK: sol.func @{{.*words.*}}
// CHECK:   sol.inline_asm {
// CHECK-DAG:     yul.constant 44048180597813453602326562734351324025098966208897425494240603688123167145984
// CHECK-DAG:     yul.constant 7749391226117993669552342838258440610419344371696464750321364798869430337536
// CHECK:     yul.add

contract C {
    function numbers() public pure returns (uint256 r) {
        assembly {
            r := add(42, 0xff)
            r := add(r, 115792089237316195423570985008687907853269984665640564039457584007913129639935)
        }
    }

    function booleans() public pure returns (uint256 r) {
        assembly {
            if true { r := 1 }
            if iszero(false) { r := 2 }
        }
    }

    function words() public pure returns (uint256 r) {
        assembly {
            r := add("abc", hex"1122")
        }
    }
}
