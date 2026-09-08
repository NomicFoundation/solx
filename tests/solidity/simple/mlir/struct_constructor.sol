//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "positional()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7",
//!                         "11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "named()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "17",
//!                         "13"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  struct S {
    uint256 a;
    uint256 b;
  }

  function positional() public pure returns (uint256, uint256) {
    S memory s = S(7, 11);
    return (s.a, s.b);
  }

  function named() public pure returns (uint256, uint256) {
    S memory s = S({b: 13, a: 17});
    return (s.a, s.b);
  }
}
