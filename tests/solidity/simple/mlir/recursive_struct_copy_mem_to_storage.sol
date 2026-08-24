//! {
//!     "modes": [
//!         "Y"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "copyMemToStorage()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "read()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7",
//!                         "2",
//!                         "8",
//!                         "9"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct S {
  uint256 a;
  S[] b;
}
contract Test {
  S s;

  function copyMemToStorage() public {
    S memory m;
    m.a = 7;
    m.b = new S[](2);
    m.b[0].a = 8;
    m.b[1].a = 9;
    s = m;
  }

  function read() public view returns (uint256, uint256, uint256, uint256) {
    return (s.a, s.b.length, s.b[0].a, s.b[1].a);
  }
}
