// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*id.*}}() -> !sol.fixedbytes<4>
// CHECK:   sol.constant 1547088262 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.return

interface I {
    event Ping(uint256 x);
    error Bad(uint256 y);
    function ping() external returns (uint256);
}

contract C {
    function id() public pure returns (bytes4) {
        return type(I).interfaceId;
    }
}
