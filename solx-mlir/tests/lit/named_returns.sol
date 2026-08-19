// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*identity.*}}
// CHECK:   %[[OUT:.*]] = sol.alloca : !sol.ptr<i1, Stack>
// CHECK:   sol.store %{{.*}}, %[[OUT]]
// CHECK:   sol.load %[[OUT]]
// CHECK:   sol.return

// CHECK: sol.func @{{.*plus_one.*}}
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   %[[SUM:.*]] = sol.cadd
// CHECK-NEXT:   sol.return %[[SUM]]

// CHECK: sol.func @{{.*named_bytes.*}}
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : ui32
// CHECK:   sol.bytes_cast %[[ZERO]] : ui32 to !sol.fixedbytes<4>

// CHECK: sol.func @{{.*named_enum.*}}
// CHECK:   %[[ORDINAL:.*]] = sol.constant 0 : ui256
// CHECK:   sol.enum_cast %[[ORDINAL]] : ui256 to !sol.enum<2>

contract C {
    enum E { First, Second, Third }

    function identity(bool _in) public pure returns (bool _out) {
        _out = _in;
    }

    function plus_one(uint256 x) public pure returns (uint256) {
        return x + 1;
    }

    function named_bytes() public pure returns (bytes4 result) {}

    function named_enum() public pure returns (E result) {}
}
