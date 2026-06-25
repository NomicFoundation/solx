// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Hex strings, unicode strings, and empty string/bytes literals all lower to a
// `sol.string_lit` carrying the raw bytes verbatim (hex"" digits become bytes,
// unicode"" is UTF-8 encoded, "" / hex"" are the empty string). Functions are
// alphabetically ordered so the solx (alphabetical) and solc (source-order)
// walks agree.

// CHECK: sol.func @{{.*}}empty_bytes
// CHECK:   sol.string_lit "" -> !sol.string<Memory>
// CHECK: sol.func @{{.*}}empty_str
// CHECK:   sol.string_lit "" -> !sol.string<Memory>
// CHECK: sol.func @{{.*}}hex_str
// CHECK:   sol.string_lit "\00\11\22" -> !sol.string<Memory>
// CHECK: sol.func @{{.*}}unicode_str
// CHECK:   sol.string_lit "h\C3\A9llo" -> !sol.string<Memory>

contract C {
    function empty_bytes() public pure returns (bytes memory) {
        return hex"";
    }
    function empty_str() public pure returns (string memory) {
        return "";
    }
    function hex_str() public pure returns (bytes memory) {
        return hex"001122";
    }
    function unicode_str() public pure returns (string memory) {
        return unicode"héllo";
    }
}
