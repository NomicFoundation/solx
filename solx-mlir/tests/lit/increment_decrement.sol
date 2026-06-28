// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*postfix_dec.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.csub %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @{{.*postfix_inc.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.cadd %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[OLD]]

// CHECK: sol.func @{{.*prefix_dec.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.csub %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @{{.*prefix_inc.*}}
// CHECK:   %[[OLD:.*]] = sol.load
// CHECK:   %[[NEW:.*]] = sol.cadd %[[OLD]]
// CHECK:   sol.store %[[NEW]]
// CHECK:   sol.return %[[NEW]]

// CHECK: sol.func @{{.*state_var_stmts.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 s;

    function postfix_dec(uint256 x) public pure returns (uint256) {
        return x--;
    }

    function postfix_inc(uint256 x) public pure returns (uint256) {
        return x++;
    }

    function prefix_dec(uint256 x) public pure returns (uint256) {
        return --x;
    }

    function prefix_inc(uint256 x) public pure returns (uint256) {
        return ++x;
    }

    function state_var_stmts(uint256 a) public {
        s++;
        s--;
        ++s;
        --s;
        a + a;
        a;
    }
}
