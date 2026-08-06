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

// CHECK: sol.func @{{.*encodeWithSelectorLiteral.*}}
// CHECK:   %[[LIT:.*]] = sol.constant 305419896 : ui32
// CHECK:   %[[LITSEL:.*]] = sol.bytes_cast %[[LIT]] : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%[[LITSEL]]) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

// CHECK: sol.func @{{.*encodeWithSignature.*}}
// CHECK:   %[[SIG:.*]] = sol.constant 801029432 : ui32
// CHECK:   %[[SIGSEL:.*]] = sol.bytes_cast %[[SIG]] : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%[[SIGSEL]]) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

// CHECK: sol.func @{{.*encodeWithSignatureDynamic.*}}
// CHECK:   %[[HASH:.*]] = "sol.keccak256"(%{{.*}}) : (!sol.string<Memory>) -> !sol.fixedbytes<32>
// CHECK:   %[[HASHSEL:.*]] = sol.bytes_cast %[[HASH]] : !sol.fixedbytes<32> to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%[[HASHSEL]]) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

// CHECK: sol.func @{{.*encodeCall.*}}
// CHECK:   %[[CALL:.*]] = sol.constant 3017696395 : ui32
// CHECK:   %[[CALLSEL:.*]] = sol.bytes_cast %[[CALL]] : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%[[CALLSEL]]) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

// CHECK: sol.func @{{.*encodeCallUnparenthesized.*}}
// CHECK:   %[[FLAT:.*]] = sol.constant 3017696395 : ui32
// CHECK:   %[[FLATSEL:.*]] = sol.bytes_cast %[[FLAT]] : ui32 to !sol.fixedbytes<4>
// CHECK:   %[[FLATARG:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.encode selector(%[[FLATSEL]]) %[[FLATARG]] : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

// CHECK: sol.func @{{.*encodeCallWidened.*}}
// CHECK:   sol.constant 3017696395 : ui32
// CHECK:   %[[NARROW:.*]] = sol.load %{{.*}} : !sol.ptr<ui8, Stack>, ui8
// CHECK:   %[[WIDE:.*]] = sol.cast %[[NARROW]] : ui8 to ui256
// CHECK:   sol.encode selector(%{{.*}}) %[[WIDE]] : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

// TODO: pin abi.encodeCall on a reference parameter. solc encodes from memory, but the parameter
// type reachable from `I.f` keeps its declared `calldata` location — slang normalizes it only on an
// instance-qualified `i.f` — so a memory or storage argument emits an unlegalizable cast.

// CHECK: sol.func @{{.*encodeCallEmpty.*}}
// CHECK:   %[[EMPTY:.*]] = sol.constant 777180678 : ui32
// CHECK:   %[[EMPTYSEL:.*]] = sol.bytes_cast %[[EMPTY]] : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%[[EMPTYSEL]]) : !sol.fixedbytes<4>  : !sol.string<Memory>

// CHECK: sol.func @{{.*encodeCallPointer.*}}
// CHECK:   %[[PTR:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.ext_func_ref<(ui256) -> ui256>, Stack>, !sol.ext_func_ref<(ui256) -> ui256>
// CHECK:   %[[PTRSEL:.*]] = sol.ext_func_selector %[[PTR]] : !sol.ext_func_ref<(ui256) -> ui256> -> !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%[[PTRSEL]]) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

// CHECK: sol.func @{{.*decode.*}}
// CHECK:   sol.decode {{.*}} : !sol.string<Memory> -> ui256

// CHECK: sol.func @{{.*decodePair.*}}
// CHECK:   sol.decode %{{.*}} : !sol.string<Memory> -> ui256, !sol.string<Memory>

// CHECK: sol.func @{{.*decodeAddress.*}}
// CHECK:   %[[PAYABLE:.*]] = sol.decode %{{.*}} : !sol.string<Memory> -> !sol.address<payable>
// CHECK:   sol.address_cast %[[PAYABLE]] : !sol.address<payable> to !sol.address

// CHECK: sol.func @{{.*decodeStorage.*}}
// CHECK:   %[[SLOT:.*]] = sol.addr_of @{{.*}} : !sol.string<Storage>
// CHECK:   %[[COPY:.*]] = sol.data_loc_cast %[[SLOT]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.decode %[[COPY]] : !sol.string<Memory> -> ui256

// CHECK: sol.func @{{.*decodeCalldata.*}}
// CHECK:   sol.decode %{{.*}} : !sol.string<CallData> -> ui256

contract C {
    bytes stored;

    function encode(uint256 x, address y) public pure returns (bytes memory) { return abi.encode(x, y); }

    function encodePacked(uint256 x, address y) public pure returns (bytes memory) { return abi.encodePacked(x, y); }

    function encodeWithSelector(bytes4 s, uint256 x) public pure returns (bytes memory) { return abi.encodeWithSelector(s, x); }

    function encodeWithSelectorTwo(bytes4 s, uint256 x, address y) public pure returns (bytes memory) { return abi.encodeWithSelector(s, x, y); }

    function encodeWithSelectorLiteral(uint256 x) public pure returns (bytes memory) { return abi.encodeWithSelector(0x12345678, x); }

    function encodeWithSignature(uint256 x) public pure returns (bytes memory) { return abi.encodeWithSignature("foo(uint256)", x); }

    function encodeWithSignatureDynamic(string memory s, uint256 x) public pure returns (bytes memory) { return abi.encodeWithSignature(s, x); }

    function encodeCall(uint256 x) public pure returns (bytes memory) { return abi.encodeCall(I.f, (x)); }

    function encodeCallUnparenthesized(uint256 x) public pure returns (bytes memory) { return abi.encodeCall(I.f, x); }

    function encodeCallWidened(uint8 x) public pure returns (bytes memory) { return abi.encodeCall(I.f, (x)); }

    function encodeCallEmpty() public pure returns (bytes memory) { return abi.encodeCall(I.n, ()); }

    function encodeCallPointer(function(uint256) external returns (uint256) p, uint256 x) public pure returns (bytes memory) { return abi.encodeCall(p, (x)); }

    function decode(bytes memory data) public pure returns (uint256) { return abi.decode(data, (uint256)); }

    function decodePair(bytes memory data) public pure returns (uint256, bytes memory) { return abi.decode(data, (uint256, bytes)); }

    function decodeAddress(bytes memory data) public pure returns (address) { return abi.decode(data, (address)); }

    function decodeStorage() public view returns (uint256) { return abi.decode(stored, (uint256)); }

    function decodeCalldata(bytes calldata data) public pure returns (uint256) { return abi.decode(data, (uint256)); }
}

interface I {
    function f(uint256 a) external returns (uint256);

    function n() external returns (uint256);
}
