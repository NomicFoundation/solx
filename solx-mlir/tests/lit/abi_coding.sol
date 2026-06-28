// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*decode[_("].*}}
// CHECK:   sol.decode {{.*}} : !sol.string<Memory> -> ui256
// CHECK: sol.func @{{.*decodeMulti[_("].*}}
// CHECK:   sol.decode %{{.*}} : !sol.string<Memory> -> ui256, i1, !sol.fixedbytes<32>
// CHECK: sol.func @{{.*encode[_("].*}}
// CHECK:   sol.encode {{.*}} : ui256, !sol.address : !sol.string<Memory>
// CHECK: sol.func @{{.*encodePacked[_("].*}}
// CHECK:   sol.encode {{.*}} : ui256, !sol.address : !sol.string<Memory> {packed}
// CHECK: sol.func @{{.*encodePackedMixed[_("].*}}
// CHECK:   sol.encode %{{.*}}, %{{.*}}, %{{.*}} :{{ +}}!sol.string<Memory>, ui8, !sol.string<Memory> : !sol.string<Memory> {packed}
// CHECK: sol.func @{{.*encodeWithSelector[_("].*}}
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>
// CHECK: sol.func @{{.*encodeWithSelectorLiteral[_("].*}}
// CHECK:   sol.constant 305419896 : ui32
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}}, %{{.*}}, %{{.*}} : !sol.fixedbytes<4> ui256, !sol.address, i1 : !sol.string<Memory>
// CHECK: sol.func @{{.*encodeWithSelectorTwo[_("].*}}
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}}, %{{.*}} : !sol.fixedbytes<4> ui256, !sol.address : !sol.string<Memory>
// CHECK: sol.func @{{.*encodeWithSignature[_("].*}}
// CHECK:   sol.constant 801029432 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

contract C {
    function decode(bytes memory data) public pure returns (uint256) { return abi.decode(data, (uint256)); }
    function decodeMulti(bytes memory data) public pure returns (uint256, bool, bytes32) { return abi.decode(data, (uint256, bool, bytes32)); }
    function encode(uint256 x, address y) public pure returns (bytes memory) { return abi.encode(x, y); }
    function encodePacked(uint256 x, address y) public pure returns (bytes memory) { return abi.encodePacked(x, y); }
    function encodePackedMixed(string memory s, uint8 n, bytes memory b) public pure returns (bytes memory) { return abi.encodePacked(s, n, b); }
    function encodeWithSelector(bytes4 s, uint256 x) public pure returns (bytes memory) { return abi.encodeWithSelector(s, x); }
    function encodeWithSelectorLiteral(uint256 x, address y, bool z) public pure returns (bytes memory) { return abi.encodeWithSelector(0x12345678, x, y, z); }
    function encodeWithSelectorTwo(bytes4 s, uint256 x, address y) public pure returns (bytes memory) { return abi.encodeWithSelector(s, x, y); }
    function encodeWithSignature(uint256 x) public pure returns (bytes memory) { return abi.encodeWithSignature("foo(uint256)", x); }
}
