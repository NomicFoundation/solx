//! {
//!     "modes": [
//!         "Y"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "emptyBytes()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32"
//!                     ]
//!                 },
//!                 {
//!                     "method": "oneByte()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "64"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fullWord()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "64"
//!                     ]
//!                 },
//!                 {
//!                     "method": "wordAndByte()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "96"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storageCopyDistance()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "64"
//!                     ]
//!                 },
//!                 {
//!                     "method": "packedDistance()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "64"
//!                     ]
//!                 },
//!                 {
//!                     "method": "integrity()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  bytes s;

  // Distance between two consecutive allocations. The allocator must reserve
  // a word-rounded span for the byte array: 32 for the length slot plus the
  // payload rounded up to a multiple of 32.
  function allocDistance(uint256 n) internal pure returns (uint256) {
    bytes memory b = new bytes(n);
    uint256[] memory a = new uint256[](1);
    uint256 bp;
    uint256 ap;
    assembly {
      bp := b
      ap := a
    }
    require(ap > bp);
    return ap - bp;
  }

  function emptyBytes() public pure returns (uint256) {
    return allocDistance(0);
  }

  function oneByte() public pure returns (uint256) {
    return allocDistance(1);
  }

  function fullWord() public pure returns (uint256) {
    return allocDistance(32);
  }

  function wordAndByte() public pure returns (uint256) {
    return allocDistance(33);
  }

  // A storage-to-memory copy of an odd-length byte array must also reserve a
  // rounded span.
  function storageCopyDistance() public returns (uint256) {
    s = hex"aabbcc";
    bytes memory m = s;
    uint256[] memory a = new uint256[](1);
    uint256 mp;
    uint256 ap;
    assembly {
      mp := m
      ap := a
    }
    require(ap > mp);
    return ap - mp;
  }

  // abi.encodePacked produces an unpadded payload; the finalized allocation
  // must still be word-rounded.
  function packedDistance() public pure returns (uint256) {
    bytes memory p = abi.encodePacked(uint8(1), uint16(2));
    uint256[] memory a = new uint256[](1);
    uint256 pp;
    uint256 ap;
    assembly {
      pp := p
      ap := a
    }
    require(ap > pp);
    return ap - pp;
  }

  // Writes to an odd-size byte array and its neighbor must not interfere.
  function integrity() public pure returns (bool) {
    bytes memory b = new bytes(5);
    uint256[] memory a = new uint256[](2);
    for (uint256 i = 0; i < 5; ++i) b[i] = 0x2a;
    a[0] = type(uint256).max;
    a[1] = 1;
    if (b.length != 5) return false;
    for (uint256 i = 0; i < 5; ++i)
      if (b[i] != 0x2a) return false;
    return a[0] == type(uint256).max && a[1] == 1;
  }
}
