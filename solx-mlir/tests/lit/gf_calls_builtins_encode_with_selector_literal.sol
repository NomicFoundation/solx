// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `abi.encodeWithSelector(0x12345678, a, b, c)` with a literal selector folds
// the selector to a `ui32` constant, casts it to `bytes4` with `sol.bytes_cast`,
// and `sol.encode`s it ahead of every remaining argument in declared order.

// CHECK: sol.constant 305419896 : ui32
// CHECK: sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK: sol.encode selector(%{{.*}}) %{{.*}}, %{{.*}}, %{{.*}} : !sol.fixedbytes<4> ui256, !sol.address, i1 : !sol.string<Memory>

contract C {
    function withSel(uint256 x, address y, bool z) public pure returns (bytes memory) {
        return abi.encodeWithSelector(0x12345678, x, y, z);
    }
}
