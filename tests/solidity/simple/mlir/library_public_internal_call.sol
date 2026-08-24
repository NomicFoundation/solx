//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "run(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

library L {
  function pub(uint x) public pure returns (uint) {
    return x * 2;
  }

  function f(uint x) internal pure returns (uint) {
    return pub(x) + 1;
  }
}

contract Test {
  function run(uint x) public pure returns (uint) {
    return L.f(x);
  }
}
