// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*variables.*}}
// CHECK:   sol.ecrecover{{.*}}: (!sol.fixedbytes<32>, ui8, !sol.fixedbytes<32>, !sol.fixedbytes<32>) -> !sol.address

// CHECK: sol.func @{{.*literals.*}}
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK:   sol.ecrecover{{.*}}: (!sol.fixedbytes<32>, ui8, !sol.fixedbytes<32>, !sol.fixedbytes<32>) -> !sol.address

// CHECK: sol.func @{{.*folded.*}}
// CHECK:   %[[MAX:.*]] = sol.constant 115792089237316195423570985008687907853269984665640564039457584007913129639935 : ui256
// CHECK:   %[[H:.*]] = sol.bytes_cast %[[MAX]] : ui256 to !sol.fixedbytes<32>
// CHECK:   %[[V:.*]] = sol.constant 1 : ui8
// CHECK:   %[[R:.*]] = sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK:   %[[S:.*]] = sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK:   sol.ecrecover"(%[[H]], %[[V]], %[[R]], %[[S]]) : (!sol.fixedbytes<32>, ui8, !sol.fixedbytes<32>, !sol.fixedbytes<32>) -> !sol.address

contract C {
    function variables(bytes32 h, uint8 v, bytes32 r, bytes32 s) public pure returns (address) {
        return ecrecover(h, v, r, s);
    }

    function literals() public pure returns (address) {
        return ecrecover(bytes32(uint256(1)), 27, bytes32(uint256(2)), bytes32(uint256(3)));
    }

    function folded() public pure returns (address) {
        return ecrecover(bytes32(type(uint256).max), 1, bytes32(uint256(2)), bytes32(uint256(3)));
    }
}
