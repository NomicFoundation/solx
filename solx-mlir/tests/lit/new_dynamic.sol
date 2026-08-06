// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*array.*}}
// CHECK:   %[[SIZE:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.malloc %[[SIZE]] zero_init : ui256 !sol.array<? x ui256, Memory>

// CHECK: sol.func @{{.*nested.*}}
// CHECK:   %[[SIZE:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.malloc %[[SIZE]] zero_init : ui256 !sol.array<? x !sol.array<? x ui256, Memory>, Memory>

// CHECK: sol.func @{{.*narrow_size.*}}
// CHECK:   %[[SIZE:.*]] = sol.load %{{.*}} : !sol.ptr<ui8, Stack>, ui8
// CHECK:   sol.malloc %[[SIZE]] zero_init : ui8 !sol.array<? x !sol.address, Memory>

// CHECK: sol.func @{{.*raw_bytes.*}}
// CHECK:   %[[SIZE:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.malloc %[[SIZE]] zero_init : ui256 !sol.string<Memory>

// CHECK: sol.func @{{.*text.*}}
// CHECK:   %[[SIZE:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.malloc %[[SIZE]] zero_init : ui256 !sol.string<Memory>

contract C {
    function array(uint256 n) public pure returns (uint256) {
        uint256[] memory values = new uint256[](n);
        return values.length;
    }

    function nested(uint256 n) public pure returns (uint256) {
        uint256[][] memory values = new uint256[][](n);
        return values.length;
    }

    function narrow_size(uint8 n) public pure returns (uint256) {
        address[] memory values = new address[](n);
        return values.length;
    }

    function raw_bytes(uint256 n) public pure returns (uint256) {
        bytes memory payload = new bytes(n);
        return payload.length;
    }

    function text(uint256 n) public pure returns (uint256) {
        string memory message = new string(n);
        return bytes(message).length;
    }
}
