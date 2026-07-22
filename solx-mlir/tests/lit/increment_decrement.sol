// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*prefix_increment.*}}
// CHECK:   %[[NEW:.*]] = sol.cadd
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @{{.*postfix_increment.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.cadd
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @{{.*prefix_decrement.*}}
// CHECK:   %[[NEW:.*]] = sol.csub
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @{{.*postfix_decrement.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.csub
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @{{.*prefix_field.*}}
// CHECK:   %[[NEW:.*]] = sol.csub
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @{{.*prefix_index.*}}
// CHECK:   %[[NEW:.*]] = sol.cadd
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @{{.*postfix_field.*}}
// CHECK:   %[[OLD:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[NEW:.*]] = sol.cadd
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @{{.*postfix_index.*}}
// CHECK:   %[[OLD:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:   %[[NEW:.*]] = sol.cadd
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @{{.*parenthesized.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.cadd
// CHECK:   sol.store %[[NEW]], %{{.*}}
// CHECK:   sol.return %[[OLD]]

contract C {
    uint256[] array;

    struct S {
        uint256 a;
    }

    S s;

    function prefix_increment(uint256 x) public pure returns (uint256) {
        return ++x;
    }

    function postfix_increment(uint256 x) public pure returns (uint256) {
        return x++;
    }

    function prefix_decrement(uint256 x) public pure returns (uint256) {
        return --x;
    }

    function postfix_decrement(uint256 x) public pure returns (uint256) {
        return x--;
    }

    function prefix_field() public returns (uint256) {
        return --s.a;
    }

    function prefix_index() public returns (uint256) {
        return ++array[0];
    }

    function postfix_field() public returns (uint256) {
        return s.a++;
    }

    function postfix_index() public returns (uint256) {
        return array[0]++;
    }

    function parenthesized(uint256 x) public pure returns (uint256) {
        return (x)++;
    }
}
