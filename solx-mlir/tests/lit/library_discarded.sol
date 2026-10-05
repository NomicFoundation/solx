// RUN: slang --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.contract @{{.*C.*}} {
// CHECK: sol.func @{{.*discarded_attached.*}}()
// CHECK-NEXT:   %[[COUNTER:.*]] = sol.addr_of @{{.*counter.*}} : !sol.ptr<ui256, Storage>
// CHECK-NEXT:   sol.load %[[COUNTER]] : !sol.ptr<ui256, Storage>, ui256
// CHECK-NEXT:   sol.return
// CHECK: sol.func @{{.*discarded_function.*}}()
// CHECK-NEXT:   sol.return
// CHECK: sol.func @{{.*discarded_library.*}}()
// CHECK-NEXT:   sol.return
// CHECK: sol.func @{{.*discarded_member.*}}()
// CHECK-NEXT:   sol.return
// CHECK: } {kind = #Contract, runtime}

// CHECK: sol.contract @{{.*Library.*}} {
// CHECK: sol.func @{{.*SEEN.*}}() -> ui256 attributes {orig_fn_type = () -> ui256, selector = -764320198 : i32, state_mutability = #Pure}
// CHECK:   sol.constant 9 : ui8
// CHECK: } {kind = #Library, runtime}

contract C {
    using Library for uint256;

    uint256 counter;

    function discarded_function() public pure {
        Library.identity;
    }

    function discarded_library() public pure {
        Library;
    }

    function discarded_member() public pure {
        Library.SEEN;
    }

    function discarded_attached() public view {
        counter.identity;
    }
}

library Library {
    uint256 public constant SEEN = 9;

    function identity(uint256 a) internal pure returns (uint256) {
        return a;
    }
}
