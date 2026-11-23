// RUN: slang --emit-mlir=sol --debug-info --debug-info-runtime %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: sol.func @{{.*decodes.*}}(
// CHECK:   sol.decode %{{.*}} loc(#[[DECODE:loc[0-9]*]])
// CHECK: sol.func @{{.*encodes.*}}(
// CHECK:   sol.encode %{{.*}} : (ui256) -> !sol.string<Memory> loc(#[[ENCODE:loc[0-9]*]])
// CHECK:   sol.encode %{{.*}} {packed} loc(#[[ENCODE_PACKED:loc[0-9]*]])
// CHECK:   sol.encode selector(%{{.*}}) {{.*}} loc(#[[ENCODE_WITH_SELECTOR:loc[0-9]*]])
// CHECK:   sol.encode selector(%{{.*}}) {{.*}} loc(#[[ENCODE_WITH_SIGNATURE:loc[0-9]*]])
// CHECK:   sol.encode selector(%{{.*}}) {{.*}} loc(#[[ENCODE_CALL:loc[0-9]*]])
// CHECK: sol.func @{{.*hashes.*}}(
// CHECK:   "sol.keccak256"(%{{.*}}) {{.*}} loc(#[[KECCAK256:loc[0-9]*]])
// CHECK:   "sol.ecrecover"(%{{.*}}) {{.*}} loc(#[[ECRECOVER:loc[0-9]*]])
// CHECK:   sol.blocknumber {{.*}} loc(#[[BLOCK_NUMBER:loc[0-9]*]])
// CHECK:   sol.blockhash %{{.*}} loc(#[[BLOCKHASH:loc[0-9]*]])
// CHECK:   sol.blobhash %{{.*}} loc(#[[BLOBHASH:loc[0-9]*]])
// CHECK: sol.func @{{.*joins.*}}(
// CHECK:   sol.concat %{{.*}} loc(#[[CONCAT:loc[0-9]*]])
// CHECK: sol.func @{{.*math.*}}(
// CHECK:   sol.assert %{{.*}} loc(#[[ASSERT:loc[0-9]*]])
// CHECK:   sol.addmod %{{.*}} loc(#[[ADDMOD:loc[0-9]*]])
// CHECK:   sol.mulmod %{{.*}} loc(#[[MULMOD:loc[0-9]*]])
// CHECK:   sol.gasleft {{.*}} loc(#[[GASLEFT:loc[0-9]*]])
// CHECK: sol.func @{{.*pops.*}}(
// CHECK:   sol.pop %{{.*}} loc(#[[POP:loc[0-9]*]])
// CHECK: sol.func @{{.*slots.*}}(
// CHECK:   "sol.erc7201"(%{{.*}}) {{.*}} loc(#[[ERC7201:loc[0-9]*]])
// CHECK: sol.func @{{.*transfers.*}}(
// CHECK:   sol.bare_call %{{.*}} loc(#[[CALL:loc[0-9]*]])
// CHECK:   sol.send %{{.*}} loc(#[[SEND:loc[0-9]*]])
// CHECK:   sol.transfer %{{.*}} loc(#[[TRANSFER:loc[0-9]*]])
// CHECK:   sol.selfdestruct %{{.*}} loc(#[[SELFDESTRUCT:loc[0-9]*]])

// CHECK-DAG: #[[DECODE]] = loc("{{.*}}debug_location_builtins.sol":90:16)
// CHECK-DAG: #[[ENCODE]] = loc("{{.*}}debug_location_builtins.sol":82:13)
// CHECK-DAG: #[[ENCODE_PACKED]] = loc("{{.*}}debug_location_builtins.sol":83:13)
// CHECK-DAG: #[[ENCODE_WITH_SELECTOR]] = loc("{{.*}}debug_location_builtins.sol":84:13)
// CHECK-DAG: #[[ENCODE_WITH_SIGNATURE]] = loc("{{.*}}debug_location_builtins.sol":85:13)
// CHECK-DAG: #[[ENCODE_CALL]] = loc("{{.*}}debug_location_builtins.sol":86:13)
// CHECK-DAG: #[[KECCAK256]] = loc("{{.*}}debug_location_builtins.sol":68:13)
// CHECK-DAG: #[[ECRECOVER]] = loc("{{.*}}debug_location_builtins.sol":69:18)
// CHECK-DAG: #[[BLOCK_NUMBER]] = loc("{{.*}}debug_location_builtins.sol":70:23)
// CHECK-DAG: #[[BLOCKHASH]] = loc("{{.*}}debug_location_builtins.sol":70:13)
// CHECK-DAG: #[[BLOBHASH]] = loc("{{.*}}debug_location_builtins.sol":71:13)
// CHECK-DAG: #[[CONCAT]] = loc("{{.*}}debug_location_builtins.sol":98:16)
// CHECK-DAG: #[[ASSERT]] = loc("{{.*}}debug_location_builtins.sol":61:9)
// CHECK-DAG: #[[ADDMOD]] = loc("{{.*}}debug_location_builtins.sol":62:13)
// CHECK-DAG: #[[MULMOD]] = loc("{{.*}}debug_location_builtins.sol":63:13)
// CHECK-DAG: #[[GASLEFT]] = loc("{{.*}}debug_location_builtins.sol":64:13)
// CHECK-DAG: #[[POP]] = loc("{{.*}}debug_location_builtins.sol":94:9)
// CHECK-DAG: #[[CALL]] = loc("{{.*}}debug_location_builtins.sol":75:20)
// CHECK-DAG: #[[SEND]] = loc("{{.*}}debug_location_builtins.sol":76:16)
// CHECK-DAG: #[[TRANSFER]] = loc("{{.*}}debug_location_builtins.sol":77:9)
// CHECK-DAG: #[[SELFDESTRUCT]] = loc("{{.*}}debug_location_builtins.sol":78:9)
// CHECK-DAG: #[[ERC7201]] = loc("{{.*}}debug_location_builtins.sol":102:16)

contract C {
    uint256[] values;

    function math(uint256 a, uint256 b) public view returns (uint256 r) {
        assert(a > b);
        r = addmod(a, b, 7);
        r = mulmod(r, b, 7);
        r = gasleft();
    }

    function hashes(bytes memory data, uint8 v, bytes32 r, bytes32 s) public view returns (bytes32 h, address signer) {
        h = keccak256(data);
        signer = ecrecover(h, v, r, s);
        h = blockhash(block.number);
        h = blobhash(0);
    }

    function transfers(address payable to) public returns (bool sent) {
        (sent, ) = to.call("");
        sent = to.send(1);
        to.transfer(1);
        selfdestruct(to);
    }

    function encodes(uint256 a) public view returns (bytes memory e) {
        e = abi.encode(a);
        e = abi.encodePacked(a);
        e = abi.encodeWithSelector(bytes4(0x12345678), a);
        e = abi.encodeWithSignature("encodes(uint256)", a);
        e = abi.encodeCall(this.encodes, (a));
    }

    function decodes(bytes memory data) public pure returns (uint256) {
        return abi.decode(data, (uint256));
    }

    function pops() public {
        values.pop();
    }

    function joins(string memory a, string memory b) public pure returns (string memory) {
        return string.concat(a, b);
    }

    function slots(string memory namespace) public pure returns (uint256) {
        return erc7201(namespace);
    }
}
