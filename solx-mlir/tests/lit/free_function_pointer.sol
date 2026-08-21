// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*invoke.*}}(%{{.*}}: !sol.func_ref<(ui256) -> ui256>, %{{.*}}: ui256) -> ui256
// CHECK: sol.func @{{.*run.*}}(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector = -1538984471 : i32
// CHECK:   %[[F:.*]] = sol.func_constant @{{.*increment.*}} : !sol.func_ref<(ui256) -> ui256>
// CHECK:   %[[R:.*]] = sol.call @{{.*invoke.*}}(%[[F]], %{{.*}}) : (!sol.func_ref<(ui256) -> ui256>, ui256) -> ui256
// CHECK:   sol.return %[[R]] : ui256

function increment(uint256 a) pure returns (uint256) {
    return a + 1;
}

contract C {
    function invoke(
        function(uint256) pure returns (uint256) f,
        uint256 x
    ) internal pure returns (uint256) {
        return f(x);
    }

    function run(uint256 x) public pure returns (uint256) {
        return invoke(increment, x);
    }
}
