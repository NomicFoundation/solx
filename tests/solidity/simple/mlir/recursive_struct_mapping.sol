//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "check()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "check()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2",
//!                         "3",
//!                         "1",
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct S {
  uint32 z;
  mapping(uint8 => S) rec;
}

contract Test {
  S data;

  function set() public {
    data.z = 2;
    S storage inner = data.rec[0];
    inner.z = 3;
    inner.rec[0].z = inner.rec[1].z + 1;
  }

  function check() public view returns (uint32, uint32, uint32, uint32) {
    return (
      data.z,
      data.rec[0].z,
      data.rec[0].rec[0].z,
      data.rec[0].rec[1].z
    );
  }
}
