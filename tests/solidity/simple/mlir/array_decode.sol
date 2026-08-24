//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "at_u8(uint8[],uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x40",
//!                         "2",
//!                         "3",
//!                         "11",
//!                         "22",
//!                         "33"
//!                     ],
//!                     "expected": [
//!                         "33"
//!                     ]
//!                 },
//!                 {
//!                     "method": "at_u32(uint32[],uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x40",
//!                         "1",
//!                         "3",
//!                         "11",
//!                         "22",
//!                         "33"
//!                     ],
//!                     "expected": [
//!                         "22"
//!                     ]
//!                 },
//!                 {
//!                     "method": "at_u64(uint64[],uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x40",
//!                         "0",
//!                         "3",
//!                         "0x1111",
//!                         "0x2222",
//!                         "0x3333"
//!                     ],
//!                     "expected": [
//!                         "0x1111"
//!                     ]
//!                 },
//!                 {
//!                     "method": "at_u128(uint128[],uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x40",
//!                         "2",
//!                         "3",
//!                         "0x1111",
//!                         "0x2222",
//!                         "0x3333"
//!                     ],
//!                     "expected": [
//!                         "0x3333"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function at_u8(uint8[] memory x, uint i) public returns (uint8) {
    return x[i];
  }

  function at_u32(uint32[] memory x, uint i) public returns (uint32) {
    return x[i];
  }

  function at_u64(uint64[] memory x, uint i) public returns (uint64) {
    return x[i];
  }

  function at_u128(uint128[] memory x, uint i) public returns (uint128) {
    return x[i];
  }
}
