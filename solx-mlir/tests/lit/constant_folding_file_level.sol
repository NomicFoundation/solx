// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A file-level (outside any contract) `constant` is folded the same way as a
// contract-level one. Here `LEN = 2 + 2` is folded to 4 and baked into the
// static array length `!sol.array<4 x ui256, Memory>` at the use site. Both
// backends agree.
//
// NOTE: solc can only fold a file-level constant when it is consumed in a
// constant context like an array size. Reading a file-level constant as a
// runtime value aborts solc (getLocalVarAddr assertion); solx handles it. That
// divergence is exercised separately, not here, to keep this test passing.

// CHECK: sol.func @{{.*read.*}}() -> ui256
// CHECK:   sol.alloca : !sol.ptr<!sol.array<4 x ui256, Memory>, Stack>
// CHECK:   sol.malloc zero_init :  !sol.array<4 x ui256, Memory>
// CHECK:   %[[A:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.array<4 x ui256, Memory>, Stack>, !sol.array<4 x ui256, Memory>
// CHECK:   %[[L:.*]] = sol.length %[[A]] : !sol.array<4 x ui256, Memory>
// CHECK:   sol.return %[[L]] : ui256

uint256 constant LEN = 2 + 2;

contract C {
    function read() public pure returns (uint256) {
        uint256[LEN] memory a;
        return a.length;
    }
}
