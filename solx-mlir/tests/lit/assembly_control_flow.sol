// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*continue_statement.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.for cond {
// CHECK:     } body {
// CHECK:       yul.if %{{.*}} {
// CHECK-NEXT:    yul.continue
// CHECK:       } else {
// CHECK-NEXT:  }
// CHECK:       yul.add %{{.*}}, %c13_i256
// CHECK:       yul.yield
// CHECK:     } step {

// CHECK: sol.func @{{.*for_statement.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[I:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %c5_i256, %[[I]] : i256, !yul.ptr
// CHECK:     yul.for cond {
// CHECK:       yul.cmp ult
// CHECK:       yul.condition
// CHECK:     } body {
// CHECK:       yul.add %{{.*}}, %c7_i256
// CHECK:       yul.yield
// CHECK:     } step {
// CHECK:       yul.add %{{.*}}, %c11_i256
// CHECK:       yul.store %{{.*}}, %[[I]] : i256, !yul.ptr
// CHECK:       yul.yield
// CHECK:     }

// CHECK: sol.func @{{.*forward_reference.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @[[LATER:.*later.*]] : (i256) -> i256 {
// CHECK:     yul.func_call @[[LATER]]

// CHECK: sol.func @{{.*if_statement.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.if %{{.*}} {
// CHECK:       yul.store %c3_i256
// CHECK-NEXT:  yul.yield
// CHECK-NEXT: } else {
// CHECK-NEXT: }

// CHECK: sol.func @{{.*infinite_for.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.for cond {
// CHECK:       %[[ONE:.*]] = yul.constant 1
// CHECK:       yul.condition %[[ONE]]
// CHECK:     } body {
// CHECK-NEXT:  yul.break
// CHECK:     } step {
// CHECK-NEXT:  yul.yield
// CHECK-NEXT: }

// CHECK: sol.func @{{.*nested_definitions.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @[[IN_IF:.*in_if.*]] : (i256) -> i256 {
// CHECK:       yul.add %{{.*}}, %c11_i256
// CHECK:     yul.func @[[IN_FOR:.*in_for.*]] : (i256) -> i256 {
// CHECK:       yul.add %{{.*}}, %c13_i256
// CHECK:     yul.func @[[IN_SWITCH:.*in_switch.*]] : (i256) -> i256 {
// CHECK:       yul.add %{{.*}}, %c17_i256
// CHECK:     yul.func @[[IN_DEFAULT:.*in_default.*]] : (i256) -> i256 {
// CHECK:       yul.add %{{.*}}, %c19_i256
// CHECK:     yul.func_call @[[IN_IF]]
// CHECK:     yul.func_call @[[IN_FOR]]
// CHECK:     yul.func_call @[[IN_SWITCH]]
// CHECK:     yul.func_call @[[IN_DEFAULT]]

// CHECK: sol.func @{{.*recursion.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @[[FACT:.*]] : (i256) -> i256 {
// CHECK:       yul.func_call @[[FACT]](%{{.*}}) : (i256) -> i256
// CHECK:     yul.func_call @[[FACT]](%{{.*}}) : (i256) -> i256

// CHECK: sol.func @{{.*switch_no_default.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.switch %{{.*}} : i256
// CHECK:     case 7 {
// CHECK:       yul.store %c29_i256
// CHECK-NEXT:  yul.yield
// CHECK-NEXT: }
// CHECK:     default {
// CHECK-NEXT:  yul.yield
// CHECK-NEXT: }

// CHECK: sol.func @{{.*switch_only_default.*}}
// CHECK:   sol.inline_asm {
// CHECK-NOT: yul.switch
// CHECK:     yul.store %c31_i256
// CHECK:   }

// CHECK: sol.func @{{.*switch_statement.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.switch %{{.*}} : i256
// CHECK:     case 0 {
// CHECK:       yul.store %c17_i256
// CHECK-NEXT:  yul.yield
// CHECK-NEXT: }
// CHECK:     case 1 {
// CHECK:       yul.store %c19_i256
// CHECK-NEXT:  yul.yield
// CHECK-NEXT: }
// CHECK:     default {
// CHECK:       yul.store %c23_i256
// CHECK-NEXT:  yul.yield
// CHECK-NEXT: }

// CHECK: sol.func @{{.*terminator_in_case.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.for cond {
// CHECK:     } body {
// CHECK:       yul.switch %{{.*}} : i256
// CHECK:       case 0 {
// CHECK-NEXT:    yul.break
// CHECK:       default {
// CHECK-NEXT:    yul.continue

// CHECK: sol.func @{{.*yul_functions.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @{{.*double.*}} : (i256) -> i256 {
// CHECK:       %[[PARAM:.*]] = yul.alloca : !yul.ptr
// CHECK:       yul.store %{{.*}}, %[[PARAM]] : i256, !yul.ptr
// CHECK:       %[[RESULT:.*]] = yul.alloca : !yul.ptr
// CHECK-NEXT:  %[[ZERO:.*]] = yul.constant 0
// CHECK-NEXT:  yul.store %[[ZERO]], %[[RESULT]] : i256, !yul.ptr
// CHECK:       yul.mul %{{.*}}, %c2_i256
// CHECK:       yul.func_return
// CHECK:     }
// CHECK:     yul.func @{{.*pair.*}} : (i256) -> (i256, i256) {
// CHECK:       yul.func_return %{{.*}}, %{{.*}} : i256, i256
// CHECK:     }
// CHECK:     yul.func @{{.*effect_only.*}} : (i256) -> () {
// CHECK:       yul.func_return{{$}}
// CHECK:     }
// CHECK:     yul.func @{{.*early.*}} : (i256) -> i256 {
// CHECK:       yul.func_return %{{.*}} : i256
// CHECK:     }
// CHECK:     %[[TWO:.*]]:2 = yul.func_call @{{.*pair.*}}(%{{.*}}) : (i256) -> (i256, i256)
// CHECK:     %[[P:.*]] = yul.alloca : !yul.ptr
// CHECK-NEXT: yul.store %[[TWO]]#0, %[[P]] : i256, !yul.ptr
// CHECK:     %[[Q:.*]] = yul.alloca : !yul.ptr
// CHECK-NEXT: yul.store %[[TWO]]#1, %[[Q]] : i256, !yul.ptr
// CHECK:     yul.func_call @{{.*effect_only.*}}(%{{.*}}) : (i256) -> ()
// CHECK:     yul.func_call @{{.*}} : (i256) -> i256
// CHECK:     yul.func_call @{{.*}} : (i256) -> i256
// CHECK:     yul.add

contract C {
    function if_statement(uint256 n) public pure returns (uint256 r) {
        assembly {
            if n { r := 3 }
        }
    }

    function for_statement(uint256 n) public pure returns (uint256 r) {
        assembly {
            for { let i := 5 } lt(i, n) { i := add(i, 11) } { r := add(r, 7) }
        }
    }

    function infinite_for() public pure returns (uint256 r) {
        assembly {
            for {} 1 {} { break }
        }
    }

    function continue_statement(uint256 n) public pure returns (uint256 r) {
        assembly {
            for { let i := 0 } lt(i, n) { i := add(i, 1) } {
                if eq(i, 1) { continue }
                r := add(r, 13)
            }
        }
    }

    function switch_statement(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 0 { r := 17 }
            case 1 { r := 19 }
            default { r := 23 }
        }
    }

    function switch_no_default(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 7 { r := 29 }
        }
    }

    function switch_only_default(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            default { r := 31 }
        }
    }

    function terminator_in_case(uint256 n) public pure returns (uint256 r) {
        assembly {
            for {} 1 {} {
                switch n
                case 0 { break }
                default { continue }
            }
        }
    }

    function yul_functions(uint256 n) public pure returns (uint256 r) {
        assembly {
            function double(x) -> y { y := mul(x, 2) }
            function pair(x) -> a, b { a := x b := add(x, 1) }
            function effect_only(x) { pop(x) }
            function early(x) -> y {
                y := x
                if iszero(x) { leave }
                y := add(y, 1)
            }
            let p, q := pair(n)
            effect_only(p)
            r := add(double(q), early(p))
        }
    }

    function forward_reference(uint256 n) public pure returns (uint256 r) {
        assembly {
            r := later(n)
            function later(x) -> y { y := add(x, 1) }
        }
    }

    function recursion(uint256 n) public pure returns (uint256 r) {
        assembly {
            function fact(k) -> y {
                y := 1
                if gt(k, 1) { y := mul(k, fact(sub(k, 1))) }
            }
            r := fact(n)
        }
    }

    function nested_definitions(uint256 n) public pure returns (uint256 r) {
        assembly {
            if 1 {
                function in_if(x) -> y { y := add(x, 11) }
                r := in_if(n)
            }
            for { let i := 0 } lt(i, 1) { i := add(i, 1) } {
                function in_for(x) -> y { y := add(x, 13) }
                r := add(r, in_for(n))
            }
            switch n
            case 0 {
                function in_switch(x) -> y { y := add(x, 17) }
                r := add(r, in_switch(n))
            }
            default {
                function in_default(x) -> y { y := add(x, 19) }
                r := add(r, in_default(n))
            }
        }
    }
}
