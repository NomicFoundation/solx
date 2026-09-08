//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "x()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "20"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "x()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "100"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint public x;

  function set(uint v) internal {
    x = v;
  }

  function f(uint v) public {
    if (v > 10) {
      return set(100);
    }
    return set(v);
  }
}
