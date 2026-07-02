// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// CHECK-SOLX: sol.func @"encode(uint256,address)"
// CHECK-SOLC: sol.func @encode_{{[0-9]+}}
// CHECK:   sol.encode {{.*}} : ui256, !sol.address : !sol.string<Memory>
// CHECK-SOLX: sol.func @"encodePacked(uint256,address)"
// CHECK-SOLC: sol.func @encodePacked_{{[0-9]+}}
// CHECK:   sol.encode {{.*}} : ui256, !sol.address : !sol.string<Memory> {packed}
// CHECK-SOLX: sol.func @"encodeWithSelector(bytes4,uint256)"
// CHECK-SOLC: sol.func @encodeWithSelector_{{[0-9]+}}
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>
// CHECK-SOLX: sol.func @"encodeWithSelectorTwo(bytes4,uint256,address)"
// CHECK-SOLC: sol.func @encodeWithSelectorTwo_{{[0-9]+}}
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}}, %{{.*}} : !sol.fixedbytes<4> ui256, !sol.address : !sol.string<Memory>
// CHECK-SOLX: sol.func @"encodeWithSignature(uint256)"
// CHECK-SOLC: sol.func @encodeWithSignature_{{[0-9]+}}
// CHECK:   sol.constant 801029432 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>
// CHECK-SOLX: sol.func @"decode(bytes)"
// CHECK-SOLC: sol.func @decode_{{[0-9]+}}
// CHECK:   sol.decode {{.*}} : !sol.string<Memory> -> ui256
// CHECK-SOLX: sol.func @"decodeMulti(bytes)"
// CHECK-SOLC: sol.func @decodeMulti_{{[0-9]+}}
// CHECK:   sol.decode %{{.*}} : !sol.string<Memory> -> ui256, i1, !sol.fixedbytes<32>

contract C {
    function encode(uint256 x, address y) public pure returns (bytes memory) { return abi.encode(x, y); }
    function encodePacked(uint256 x, address y) public pure returns (bytes memory) { return abi.encodePacked(x, y); }
    function encodeWithSelector(bytes4 s, uint256 x) public pure returns (bytes memory) { return abi.encodeWithSelector(s, x); }
    function encodeWithSelectorTwo(bytes4 s, uint256 x, address y) public pure returns (bytes memory) { return abi.encodeWithSelector(s, x, y); }
    function encodeWithSignature(uint256 x) public pure returns (bytes memory) { return abi.encodeWithSignature("foo(uint256)", x); }
    function decode(bytes memory data) public pure returns (uint256) { return abi.decode(data, (uint256)); }
    function decodeMulti(bytes memory data) public pure returns (uint256, bool, bytes32) { return abi.decode(data, (uint256, bool, bytes32)); }
}
