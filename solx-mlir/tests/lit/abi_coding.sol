// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*encode.*}}
// CHECK:   sol.encode {{.*}} : ui256, !sol.address : !sol.string<Memory>
// CHECK: sol.func @{{.*encodePacked.*}}
// CHECK:   sol.encode {{.*}} : ui256, !sol.address : !sol.string<Memory> {packed}
// CHECK: sol.func @{{.*encodeWithSelector.*}}
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>
// CHECK: sol.func @{{.*encodeWithSelectorTwo.*}}
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}}, %{{.*}} : !sol.fixedbytes<4> ui256, !sol.address : !sol.string<Memory>
// CHECK: sol.func @{{.*encodeWithSignature.*}}
// CHECK:   sol.constant 801029432 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>
// CHECK: sol.func @{{.*decode.*}}
// CHECK:   sol.decode {{.*}} : !sol.string<Memory> -> ui256

contract C {
    function encode(uint256 x, address y) public pure returns (bytes memory) { return abi.encode(x, y); }
    function encodePacked(uint256 x, address y) public pure returns (bytes memory) { return abi.encodePacked(x, y); }
    function encodeWithSelector(bytes4 s, uint256 x) public pure returns (bytes memory) { return abi.encodeWithSelector(s, x); }
    function encodeWithSelectorTwo(bytes4 s, uint256 x, address y) public pure returns (bytes memory) { return abi.encodeWithSelector(s, x, y); }
    function encodeWithSignature(uint256 x) public pure returns (bytes memory) { return abi.encodeWithSignature("foo(uint256)", x); }
    function decode(bytes memory data) public pure returns (uint256) { return abi.decode(data, (uint256)); }
}
