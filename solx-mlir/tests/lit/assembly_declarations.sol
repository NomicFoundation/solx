// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// An uninitialized `let` defaults to zero.
// CHECK: sol.func @{{.*uninitialized.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[Z:.*]] = yul.constant 0
// CHECK:     %[[X:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[Z]], %[[X]] : i256, !yul.ptr

// A multi-variable `let` materializes every value before allocating any slot, so both
// zeroes precede both allocas.
// CHECK: sol.func @{{.*uninitialized_tuple.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[Z1:.*]] = yul.constant 0
// CHECK:     %[[Z2:.*]] = yul.constant 0
// CHECK:     %[[X:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[Z1]], %[[X]] : i256, !yul.ptr
// CHECK:     %[[Y:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[Z2]], %[[Y]] : i256, !yul.ptr

// A multi-return call binds one slot per result, and assigning to several paths at once
// writes each through its own store.
// CHECK: sol.func @{{.*tuple_assignment.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[FIRST:.*]]:2 = yul.func_call @{{.*pair.*}}
// CHECK:     %[[P:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[FIRST]]#0, %[[P]] : i256, !yul.ptr
// CHECK:     %[[Q:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[FIRST]]#1, %[[Q]] : i256, !yul.ptr
// CHECK:     %[[SECOND:.*]]:2 = yul.func_call @{{.*pair.*}}
// CHECK:     yul.store %[[SECOND]]#0, %[[P]] : i256, !yul.ptr
// CHECK:     yul.store %[[SECOND]]#1, %[[Q]] : i256, !yul.ptr

// A nested block opens no region: Yul's block scoping is already resolved per declaration,
// so a name reused in a sibling block is simply a second slot.
// CHECK: sol.func @{{.*nested_blocks.*}}
// CHECK:   sol.inline_asm {
// CHECK-NOT: yul.scope
// CHECK:     %[[ONE:.*]] = yul.constant 1
// CHECK:     %[[A:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[ONE]], %[[A]] : i256, !yul.ptr
// CHECK:     %[[TWO:.*]] = yul.constant 2
// CHECK:     %[[B:.*]] = yul.alloca : !yul.ptr
// CHECK:     yul.store %[[TWO]], %[[B]] : i256, !yul.ptr

contract C {
    function uninitialized() public pure returns (uint256 r) {
        assembly {
            let x
            r := x
        }
    }

    function uninitialized_tuple() public pure returns (uint256 r) {
        assembly {
            let x, y
            r := add(x, y)
        }
    }

    function tuple_assignment(uint256 n) public pure returns (uint256 r) {
        assembly {
            function pair(x) -> a, b { a := x b := add(x, 1) }
            let p, q := pair(n)
            p, q := pair(q)
            r := add(p, q)
        }
    }

    function nested_blocks() public pure returns (uint256 r) {
        assembly {
            { let x := 1 r := x }
            { let x := 2 r := add(r, x) }
        }
    }
}
