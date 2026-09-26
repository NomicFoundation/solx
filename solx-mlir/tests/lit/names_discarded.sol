// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*errors.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*events.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*structs.*}}()
// CHECK-NEXT:   sol.return

struct Pair {
    uint256 first;
}

error Missing(uint256 code);

event Logged(uint256 code);

contract C {
    struct Inner {
        uint256 value;
    }

    error Absent(uint256 code);

    event Traced(uint256 code);

    function structs() public pure {
        Pair;
        Inner;
    }

    function errors() public pure {
        Missing;
        Absent;
    }

    function events() public pure {
        Logged;
        Traced;
    }
}
