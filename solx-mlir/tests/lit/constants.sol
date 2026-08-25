// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*read.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 42 : ui8
// CHECK:   %{{.*}} = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.return %{{.*}} : ui256

// CHECK:      sol.func @{{.*sum.*}}() -> ui256
// CHECK-DAG:    sol.constant 42 : ui8
// CHECK-DAG:    sol.constant 42 : ui8
// CHECK:        sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:        sol.return %{{.*}} : ui256

// CHECK:      sol.func @{{.*getDouble.*}}() -> ui8
// CHECK-DAG:    sol.constant 42 : ui8
// CHECK-DAG:    sol.constant 2 : ui8
// CHECK:        sol.cmul %{{.*}}, %{{.*}} : ui8

// CHECK: sol.func @{{.*digits.*}}() -> !sol.fixedbytes<16>
// CHECK:   %[[DGC:.*]] = sol.constant 64058384521018188869745042196707698022 : ui128
// CHECK:   %[[DG:.*]] = sol.bytes_cast %[[DGC]] : ui128 to !sol.fixedbytes<16>
// CHECK:   sol.return %[[DG]] : !sol.fixedbytes<16>

// CHECK: sol.func @{{.*hexDigit.*}} -> !sol.fixedbytes<1>
// CHECK:   %[[HDC:.*]] = sol.constant 64058384521018188869745042196707698022 : ui128
// CHECK:   %[[HD:.*]] = sol.bytes_cast %[[HDC]] : ui128 to !sol.fixedbytes<16>
// CHECK:   sol.fixed_bytes_index %[[HD]][%{{.*}}] : !sol.fixedbytes<16>, ui256 -> !sol.fixedbytes<1>

// CHECK: sol.func @{{.*text.*}}() -> !sol.string<Memory>
// CHECK:   %[[TX:.*]] = sol.string_lit "solx" -> !sol.string<Memory>
// CHECK:   sol.return %[[TX]] : !sol.string<Memory>

// CHECK: sol.func @{{.*fileDigit.*}} -> !sol.fixedbytes<1>
// CHECK:   %[[FDC:.*]] = sol.constant 92071172066227433993572960279997788208 : ui128
// CHECK:   %[[FD:.*]] = sol.bytes_cast %[[FDC]] : ui128 to !sol.fixedbytes<16>
// CHECK:   sol.fixed_bytes_index %[[FD]][%{{.*}}] : !sol.fixedbytes<16>, ui256 -> !sol.fixedbytes<1>

bytes16 constant FILE_DIGITS = "EDCBA98765432100";

contract C {
    uint256 constant FOO = 42;
    uint8 constant DOUBLE = uint8(FOO) * 2;

    function read() public pure returns (uint256) {
        return FOO;
    }

    function sum() public pure returns (uint256) {
        return FOO + FOO;
    }

    function getDouble() public pure returns (uint8) {
        return DOUBLE;
    }

    bytes16 private constant HEX_DIGITS = "0123456789abcdef";
    string constant public TEXT = "solx";

    function digits() public pure returns (bytes16) {
        return HEX_DIGITS;
    }

    function hexDigit(uint256 value) public pure returns (bytes1) {
        return HEX_DIGITS[value & 0xf];
    }

    function text() public pure returns (string memory) {
        return TEXT;
    }

    function fileDigit(uint256 value) public pure returns (bytes1) {
        return FILE_DIGITS[value & 0xf];
    }
}
