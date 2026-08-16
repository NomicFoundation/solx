// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @{{.*Lib.*}} {
// CHECK: sol.func @{{.*external_member.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, orig_fn_type = (ui256) -> ui256, selector = -1958955763 : i32, state_mutability = #Pure}
// CHECK:   sol.call @{{.*inner.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*inner.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability = #Pure}
// CHECK:   sol.call @{{.*shared.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: } {kind = #Library}

function shared(uint256 a) pure returns (uint256) {
    return a + 1;
}

library Lib {
    uint256 internal constant FACTOR = 2;

    function external_member(uint256 a) external pure returns (uint256) {
        return inner(a);
    }

    function inner(uint256 a) internal pure returns (uint256) {
        return shared(a) * FACTOR;
    }
}
