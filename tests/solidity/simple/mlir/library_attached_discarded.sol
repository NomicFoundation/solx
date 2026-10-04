//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f()",
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

library L {
  function twice(uint v) internal pure returns (uint) {
    return 2 * v;
  }
}

contract Test {
  using L for uint;

  uint x;

  function bump() internal returns (uint) {
    return ++x;
  }

  function f() public returns (uint) {
    bump().twice;
    return x;
  }
}
