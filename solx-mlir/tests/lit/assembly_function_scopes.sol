// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*sibling_scopes.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @[[G1:g_[0-9]+]] : () -> i256 {
// CHECK:       yul.store %c1_i256
// CHECK:     yul.func @[[G2:g_[0-9]+]] : () -> i256 {
// CHECK:       yul.store %c2_i256
// CHECK-NOT: yul.func @[[G1]]
// CHECK-NOT: yul.func @[[G2]]
// CHECK:     yul.func_call @[[G1]]() : () -> i256
// CHECK:     yul.func_call @[[G2]]() : () -> i256

// CHECK: sol.func @{{.*leave_in_for_init.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @{{.*g.*}} : () -> i256 {
// CHECK-NOT:   yul.for
// CHECK:       yul.func_return

// CHECK: sol.func @{{.*leave_in_for_step.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @{{.*h.*}} : (i256) -> i256 {
// CHECK:       yul.for cond {
// CHECK:       } body {
// CHECK:       } step {
// CHECK:         yul.func_return

// CHECK: sol.func @{{.*terminator_in_block.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @{{.*}} : () -> i256 {
// CHECK:       yul.store %c41_i256
// CHECK-NOT:   yul.constant 43
// CHECK:       yul.func_return

// CHECK: sol.func @{{.*terminator_in_case.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.switch %{{.*}} : i256
// CHECK:     case 0 {
// CHECK-NEXT:  yul.break
// CHECK-NEXT: }
// CHECK-NOT:   yul.constant 47

contract C {
    function sibling_scopes() public pure returns (uint256 r) {
        assembly {
            { function g() -> y { y := 1 } r := g() }
            { function g() -> y { y := 2 } r := add(r, g()) }
        }
    }

    function leave_in_for_init() public pure returns (uint256 r) {
        assembly {
            function g() -> y { for { y := 1 leave } 1 {} {} }
            r := g()
        }
    }

    function leave_in_for_step(uint256 n) public pure returns (uint256 r) {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { leave } { y := add(y, 1) } }
            r := h(n)
        }
    }

    function terminator_in_block() public pure returns (uint256 r) {
        assembly {
            function g() -> y {
                {
                    y := 41
                    leave
                }
                y := 43
            }
            r := g()
        }
    }

    function terminator_in_case(uint256 n) public pure returns (uint256 r) {
        assembly {
            for {} 1 {} {
                switch n
                case 0 {
                    break
                    r := 47
                }
                default { continue }
            }
        }
    }
}
