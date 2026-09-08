//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "dynStringArray()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dynBytesArray()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fixedStringArray()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "writeOneElement()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "2",
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  mapping(uint => uint) m;

  // The mapping write leaves its key in scratch memory [0x00, 0x20). If the
  // element slots were zero-initialized to 0 instead of the 0x60 empty-string
  // sentinel, the length reads below would return the stale scratch contents.
  function dirtyScratch() internal {
    m[123] = 1;
  }

  function dynStringArray() public returns (uint) {
    dirtyScratch();
    string[] memory s = new string[](2);
    return bytes(s[0]).length;
  }

  function dynBytesArray() public returns (uint) {
    dirtyScratch();
    bytes[] memory b = new bytes[](2);
    return b[1].length;
  }

  function fixedStringArray() public returns (uint) {
    dirtyScratch();
    string[3] memory a;
    return bytes(a[2]).length;
  }

  // Writing one element must not disturb the empty neighbors.
  function writeOneElement() public returns (uint, uint, uint) {
    dirtyScratch();
    string[] memory s = new string[](3);
    s[1] = "hi";
    return (bytes(s[0]).length, bytes(s[1]).length, bytes(s[2]).length);
  }
}
