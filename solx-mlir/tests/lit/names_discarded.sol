// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init crashes on a discarded struct, error, or event name, so this is solx-only.

// CHECK: sol.func @{{.*structs.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*errors.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*events.*}}()
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
