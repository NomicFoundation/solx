//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "encodePackedStructNeighbor()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodePackedDynArrayNeighbor()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "300"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodePackedPayload()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "6",
//!                         "0x0100030405060000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodeWithSelectorDynArrayNeighbor()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "300"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodeWithSignatureDynArrayNeighbor()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "300"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodeCallDynArrayNeighbor()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "300"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodeDynArrayNeighbor()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "300"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodeFarFromEnd()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.12;

// Companion to memory_bytes_delete_element_concat_neighbor.sol for byte arrays produced by the
// `abi.encode*` functions. Under evmasm, the ABI encoder advances the free memory pointer by exactly
// the encoded size and does not round up to a 32-byte boundary, so the next object starts right
// after the last byte (E = D+L, P = 0) and a full-word write from index i reaches it for every
// i >= L-31. Note that this includes `abi.encode`, whose result length is a multiple of 32 and
// therefore never has padding, whichever allocator is used.
interface I { function foo(uint8 a) external; }
contract Test {
    struct S { uint256 x; }

    // L=2, i=1, so the top 31 bytes of s.x are in reach
    function encodePackedStructNeighbor() external pure returns (uint256) {
        bytes memory b = abi.encodePacked(uint8(1), uint8(2));
        S memory s = S(type(uint256).max);
        delete b[1];
        return s.x;
    }

    // L=2, i=1, so 31 bytes of the neighbor's length word are in reach and only the low byte of 300 = 0x012c survives
    function encodePackedDynArrayNeighbor() external pure returns (uint256) {
        bytes memory b = abi.encodePacked(uint8(1), uint8(2));
        uint256[] memory a = new uint256[](300);
        delete b[1];
        return a.length;
    }

    // L=6, i=1, so bytes 2..5 of the payload are zeroed as well
    function encodePackedPayload() external pure returns (bytes memory b) {
        b = abi.encodePacked(hex"0102", hex"03040506");
        delete b[1];
    }

    // L=36, i=35
    function encodeWithSelectorDynArrayNeighbor() external pure returns (uint256) {
        bytes memory b = abi.encodeWithSelector(I.foo.selector, uint8(1));
        uint256[] memory a = new uint256[](300);
        delete b[35];
        return a.length;
    }

    // L=36, i=35
    function encodeWithSignatureDynArrayNeighbor() external pure returns (uint256) {
        bytes memory b = abi.encodeWithSignature("foo(uint8)", uint8(1));
        uint256[] memory a = new uint256[](300);
        delete b[35];
        return a.length;
    }

    // L=36, i=35
    function encodeCallDynArrayNeighbor() external pure returns (uint256) {
        bytes memory b = abi.encodeCall(I.foo, (uint8(1)));
        uint256[] memory a = new uint256[](300);
        delete b[35];
        return a.length;
    }

    // L=64, i=63
    function encodeDynArrayNeighbor() external pure returns (uint256) {
        bytes memory b = abi.encode(uint8(1), uint8(2));
        uint256[] memory a = new uint256[](300);
        delete b[63];
        return a.length;
    }

    // L=64, i=33=L-31, so exactly the top byte of s.x is in reach
    function encodeFarFromEnd() external pure returns (uint256) {
        bytes memory b = abi.encode(uint8(1), uint8(2));
        S memory s = S(type(uint256).max);
        delete b[33];
        return s.x;
    }
}
