// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Named return: `_out` gets a zero-initialized slot, the body assigns to it,
// and the implicit return at the end loads from the slot.
// CHECK: sol.func @{{.*identity.*}}
// CHECK:   %[[IN:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   sol.store %{{.*}}, %[[IN]]
// CHECK:   %[[OUT:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   sol.store %{{.*}}, %[[OUT]]
// CHECK:   sol.load %[[IN]]
// CHECK:   sol.store %{{.*}}, %[[OUT]]
// CHECK:   sol.load %[[OUT]]
// CHECK:   sol.return

// Unnamed return: no slot allocation; only the parameter alloca.
// CHECK-LABEL: sol.func @{{.*plus_one.*}}
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK-NOT: sol.alloca
// CHECK: sol.return

contract C {
    function identity(bool _in) public pure returns (bool _out) {
        _out = _in;
    }

    function plus_one(uint256 x) public pure returns (uint256) {
        return x + 1;
    }
}
