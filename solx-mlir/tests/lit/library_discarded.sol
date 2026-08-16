// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A discarded library-rooted expression evaluates nothing: solc's print-init still materialises
// the linked address for a bare library name, so this is solx-only.

// CHECK: sol.contract @{{.*C.*}} {
// CHECK: sol.func @{{.*discarded_function.*}}()
// CHECK-NEXT:   sol.return
// CHECK: sol.func @{{.*discarded_library.*}}()
// CHECK-NEXT:   sol.return
// CHECK: sol.func @{{.*discarded_member.*}}()
// CHECK-NEXT:   sol.return
// CHECK: } {kind = #Contract}

// CHECK: sol.contract @{{.*Library.*}} {
// CHECK: sol.func @{{.*SEEN.*}}() -> ui256 attributes {orig_fn_type = () -> ui256, selector = -764320198 : i32, state_mutability = #Pure}
// CHECK:   sol.constant 9 : ui8
// CHECK: } {kind = #Library}

contract C {
    function discarded_function() public pure {
        Library.identity;
    }

    function discarded_library() public pure {
        Library;
    }

    function discarded_member() public pure {
        Library.SEEN;
    }
}

library Library {
    uint256 public constant SEEN = 9;

    function identity(uint256 a) internal pure returns (uint256) {
        return a;
    }
}
