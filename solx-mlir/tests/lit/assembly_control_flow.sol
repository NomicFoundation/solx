// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Yul has no `else`, so the op's else region stays blockless.
// CHECK: sol.func @{{.*if_statement.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.if %{{.*}} {
// CHECK:       yul.store
// CHECK:       yul.yield
// CHECK:     } else {
// CHECK-NEXT: }

// A Yul `for` initializer declares into the enclosing block, not into a region.
// CHECK: sol.func @{{.*for_statement.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[I:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.for cond {
// CHECK:       yul.condition
// CHECK:     } body {
// CHECK:       yul.yield
// CHECK:     } step {
// CHECK:       yul.yield
// CHECK:     }

// CHECK: sol.func @{{.*infinite_for.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.for cond {
// CHECK:       %[[ONE:.*]] = yul.constant 1
// CHECK:       yul.condition %[[ONE]]
// CHECK:     } body {
// CHECK:       yul.break
// CHECK:     } step {
// CHECK:       yul.yield
// CHECK:     }

// CHECK: sol.func @{{.*continue_statement.*}}
// CHECK:   sol.inline_asm {
// CHECK:       yul.continue

// CHECK: sol.func @{{.*switch_statement.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.switch %{{.*}} : i256
// CHECK:     case 0 {
// CHECK:       yul.yield
// CHECK:     }
// CHECK:     case 1 {
// CHECK:       yul.yield
// CHECK:     }
// CHECK:     default {
// CHECK:       yul.yield
// CHECK:     }

// A switch with no default still gets the op's mandatory default region, empty.
// CHECK: sol.func @{{.*switch_no_default.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.switch %{{.*}} : i256
// CHECK:     case 7 {
// CHECK:       yul.store
// CHECK:       yul.yield
// CHECK:     }
// CHECK:     default {
// CHECK-NEXT:  yul.yield
// CHECK-NEXT: }

// A switch with only a default is no switch at all: the argument is evaluated for its
// effects and the body emitted inline.
// CHECK: sol.func @{{.*switch_only_default.*}}
// CHECK:   sol.inline_asm {
// CHECK-NOT: yul.switch
// CHECK:     yul.store
// CHECK:   }

// Every Yul function is emitted ahead of the body it is written in.
// CHECK: sol.func @{{.*yul_functions.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @{{.*double.*}} : (i256) -> i256 {
// CHECK:       %[[PARAM:.*]] = yul.alloca : !yul.ptr
// CHECK:       yul.store %{{.*}}, %[[PARAM]] : i256, !yul.ptr
// CHECK:       %[[RESULT:.*]] = yul.alloca : !yul.ptr
// CHECK:       yul.func_return
// CHECK:     }
// CHECK:     yul.func @{{.*pair.*}} : (i256) -> (i256, i256) {
// CHECK:       yul.func_return %{{.*}}, %{{.*}} : i256, i256
// CHECK:     }
// CHECK:     yul.func @{{.*effect_only.*}} : (i256) -> () {
// CHECK:       yul.func_return{{$}}
// CHECK:     }
// CHECK:     yul.func @{{.*early.*}} : (i256) -> i256 {
// A `leave` is an early func_return carrying the return variables as they stand.
// CHECK:       yul.func_return %{{.*}} : i256
// CHECK:     }
// CHECK:     %[[TWO:.*]]:2 = yul.func_call @{{.*pair.*}}(%{{.*}}) : (i256) -> (i256, i256)
// CHECK:     yul.func_call @{{.*effect_only.*}}(%{{.*}}) : (i256) -> ()
// The two calls feeding `add` are asserted orderlessly; evaluation order is pinned in
// assembly_evaluation_order.sol.
// CHECK-DAG:     yul.func_call @{{.*double.*}}(%{{.*}}) : (i256) -> i256
// CHECK-DAG:     yul.func_call @{{.*early.*}}(%{{.*}}) : (i256) -> i256

// A call may name a function defined after it.
// CHECK: sol.func @{{.*forward_reference.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @{{.*later.*}}
// CHECK:     yul.func_call @{{.*later.*}}

// A Yul function may call itself.
// CHECK: sol.func @{{.*recursion.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @[[FACT:.*]] : (i256) -> i256 {
// CHECK:       yul.func_call @[[FACT]](%{{.*}}) : (i256) -> i256
// CHECK:     yul.func_call @[[FACT]](%{{.*}}) : (i256) -> i256

// A function declared in an `if` / `for` / `switch` body is scoped to that body, and all of
// them hoist to the top of the region in pre-order.
// CHECK: sol.func @{{.*nested_definitions.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @[[IN_IF:.*in_if.*]] : (i256) -> i256 {
// CHECK:     yul.func @[[IN_FOR:.*in_for.*]] : (i256) -> i256 {
// A function declared in a case body hoists ahead of one declared in the default body.
// CHECK:     yul.func @[[IN_SWITCH:.*in_switch.*]] : (i256) -> i256 {
// CHECK:     yul.func @[[IN_DEFAULT:.*in_default.*]] : (i256) -> i256 {
// CHECK:     yul.func_call @[[IN_IF]]
// CHECK:     yul.func_call @[[IN_FOR]]
// CHECK:     yul.func_call @[[IN_SWITCH]]
// CHECK:     yul.func_call @[[IN_DEFAULT]]

contract C {
    function if_statement(uint256 n) public pure returns (uint256 r) {
        assembly {
            if n { r := 1 }
        }
    }

    function for_statement(uint256 n) public pure returns (uint256 r) {
        assembly {
            for { let i := 0 } lt(i, n) { i := add(i, 1) } { r := add(r, i) }
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
                r := add(r, i)
            }
        }
    }

    function switch_statement(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 0 { r := 10 }
            case 1 { r := 11 }
            default { r := 12 }
        }
    }

    function switch_no_default(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 7 { r := 13 }
        }
    }

    function switch_only_default(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            default { r := 14 }
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
                function in_if(x) -> y { y := add(x, 1) }
                r := in_if(n)
            }
            for { let i := 0 } lt(i, 1) { i := add(i, 1) } {
                function in_for(x) -> y { y := add(x, 2) }
                r := add(r, in_for(n))
            }
            switch n
            case 0 {
                function in_switch(x) -> y { y := add(x, 3) }
                r := add(r, in_switch(n))
            }
            default {
                function in_default(x) -> y { y := add(x, 4) }
                r := add(r, in_default(n))
            }
        }
    }
}
