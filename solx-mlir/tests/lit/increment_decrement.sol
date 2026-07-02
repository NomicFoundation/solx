// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*postfix_decrement.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.csub %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @{{.*postfix_increment.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.cadd %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @{{.*prefix_decrement.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.csub %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @{{.*prefix_increment.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.cadd %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @{{.*state_variable_statements.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 s;

    function postfix_decrement(uint256 x) public pure returns (uint256) {
        return x--;
    }

    function postfix_increment(uint256 x) public pure returns (uint256) {
        return x++;
    }

    function prefix_decrement(uint256 x) public pure returns (uint256) {
        return --x;
    }

    function prefix_increment(uint256 x) public pure returns (uint256) {
        return ++x;
    }

    function state_variable_statements(uint256 a) public {
        s++;
        s--;
        ++s;
        --s;
        a + a;
        a;
    }
}
