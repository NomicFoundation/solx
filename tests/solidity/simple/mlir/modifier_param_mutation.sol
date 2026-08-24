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
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "50"
//!                     ],
//!                     "expected": [
//!                         "50"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  modifier clamp(uint v) {
    if (v > 10) v = 10;
    _;
  }

  function f(uint v) public clamp(v) returns (uint) {
    return v;
  }
}
