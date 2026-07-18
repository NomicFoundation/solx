// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*address_literal.*}}
// CHECK:   sol.constant 255 : ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK: sol.func @{{.*neg_int8.*}}
// CHECK:   %[[N:.*]] = sol.constant -1 : si8
// CHECK:   sol.return %[[N]] : si8

// CHECK: sol.func @{{.*neg_int8_min.*}}
// CHECK:   sol.constant -128 : si8

// CHECK: sol.func @{{.*ether_rational.*}}
// CHECK:   sol.constant 500000000000000000 : ui64

// CHECK: sol.func @{{.*scientific.*}}
// CHECK:   sol.constant 1000000000000000000 : ui64

// CHECK: sol.func @{{.*hex_to_bytes1.*}}
// CHECK:   sol.constant 18 : ui8
// CHECK:   sol.bytes_cast %{{.*}} : ui8 to !sol.fixedbytes<1>

// CHECK: sol.func @{{.*hex_to_bytes32.*}}
// CHECK:   sol.constant 0 : ui8
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>

// CHECK: sol.func @{{.*hex_to_bytes4.*}}
// CHECK:   sol.constant 2864434397 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

// CHECK: sol.func @{{.*string_to_bytes32.*}}
// CHECK:   sol.constant 47219736118171679016481614208494153725245902603978864281390662590579859259392 : ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>

// CHECK: sol.func @{{.*string_literal_to_bytes32.*}}
// CHECK:   sol.constant 44048180597813453602326562734351324025098966208897425494240603688123167145984 : ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>

// CHECK: sol.func @{{.*hex_string_to_bytes2.*}}
// CHECK:   sol.constant 65281 : ui16
// CHECK:   sol.bytes_cast %{{.*}} : ui16 to !sol.fixedbytes<2>

// CHECK: sol.func @{{.*hex_string_to_bytes_memory.*}}
// CHECK:   sol.string_lit "\FF\01" -> !sol.string<Memory>

contract C {
    function address_literal() public pure returns (address) {
        return 0x00000000000000000000000000000000000000ff;
    }

    function neg_int8() public pure returns (int8) {
        return -1;
    }

    function neg_int8_min() public pure returns (int8) {
        return -128;
    }

    function ether_rational() public pure returns (uint256) {
        return 0.5 ether;
    }

    function scientific() public pure returns (uint256) {
        return 1e18;
    }

    function hex_to_bytes1() public pure returns (bytes1) {
        return 0x12;
    }

    function hex_to_bytes32() public pure returns (bytes32) {
        return 0x0;
    }

    function hex_to_bytes4() public pure returns (bytes4) {
        return 0xaabbccdd;
    }

    function string_to_bytes32() public pure returns (bytes32) {
        bytes32 x = "hello";
        return x;
    }

    function string_literal_to_bytes32() public pure returns (bytes32) {
        return "abc";
    }

    function hex_string_to_bytes2() public pure returns (bytes2) {
        return hex"ff01";
    }

    function hex_string_to_bytes_memory() public pure returns (bytes memory) {
        return hex"ff01";
    }
}
