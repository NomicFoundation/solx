// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*booleans.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.if %c1_i256 {
// CHECK:     %[[FALSE:.*]] = yul.constant 0
// CHECK:     %[[ZERO:.*]] = yul.constant 0
// CHECK:     yul.cmp eq, %[[FALSE]], %[[ZERO]]
// CHECK:     yul.if

// CHECK: sol.func @{{.*numbers.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.add %c42_i256, %c255_i256
// CHECK:     yul.add %{{.*}}, %c-1_i256

// CHECK: sol.func @{{.*words.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.add %c44048180597813453602326562734351324025098966208897425494240603688123167145984_i256, %c7749391226117993669552342838258440610419344371696464750321364798869430337536_i256

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
