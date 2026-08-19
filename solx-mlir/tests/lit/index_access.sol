// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}readArray{{.*}}-> ui256
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>, ui256, !sol.ptr<ui256, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Memory>, ui256

// CHECK: sol.func {{.*}}readBytes{{.*}}-> !sol.fixedbytes<1>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.string<Memory>, ui256, !sol.ptr<!sol.byte, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<!sol.byte, Memory>, !sol.byte
// CHECK:   sol.bytes_cast %{{.*}} : !sol.byte to !sol.fixedbytes<1>

// CHECK: sol.func {{.*}}readFixedBytes{{.*}}-> !sol.fixedbytes<1>
// CHECK:   sol.fixed_bytes_index %{{.*}}[%{{.*}}] : !sol.fixedbytes<32>, ui256 -> !sol.fixedbytes<1>

// CHECK: sol.func {{.*}}readMapping{{.*}}-> ui256
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<ui256, ui256>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256

// CHECK: sol.func {{.*}}writeArray
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>, ui256, !sol.ptr<ui256, Memory>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Memory>

// CHECK: sol.func {{.*}}writeMapping
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<ui256, ui256>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func {{.*}}readLiteralKey{{.*}}-> ui256
// CHECK:   sol.constant 44048180597813453602326562734351324025098966208897425494240603688123167145984 : ui256
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.fixedbytes<32>, ui256>, !sol.fixedbytes<32>, !sol.ptr<ui256, Storage>

// CHECK: sol.func {{.*}}readNarrowKey{{.*}}-> ui256
// CHECK:   sol.cast %{{.*}} : si8 to si256
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<si256, ui256>, si256, !sol.ptr<ui256, Storage>

contract C {
    mapping(uint256 => uint256) map;
    mapping(bytes32 => uint256) wordMap;
    mapping(int256 => uint256) signedMap;

    function readArray(uint256[] memory array, uint256 index) public pure returns (uint256) {
        return array[index];
    }

    function readBytes(bytes memory data, uint256 index) public pure returns (bytes1) {
        return data[index];
    }

    function readFixedBytes(bytes32 word, uint256 index) public pure returns (bytes1) {
        return word[index];
    }

    function readMapping(uint256 key) public view returns (uint256) {
        return map[key];
    }

    function writeArray(uint256[] memory array, uint256 index, uint256 value) public pure {
        array[index] = value;
    }

    function writeMapping(uint256 key, uint256 value) public {
        map[key] = value;
    }

    function readLiteralKey() public view returns (uint256) {
        return wordMap["abc"];
    }

    function readNarrowKey(int8 key) public view returns (uint256) {
        return signedMap[key];
    }
}
